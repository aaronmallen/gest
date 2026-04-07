use std::{collections::HashMap, path::Path, time::SystemTime};

use chrono::Utc;
use libsql::{Connection, Value as DbValue};
use serde_json::Value;

use super::Error;
use crate::store::model::primitives::Id;

/// Columns that are foreign keys and must not be updated via generic upsert.
const FK_COLUMNS: &[&str] = &["assigned_to", "author_id", "entity_id", "project_id", "tag_id"];

/// Columns that should be skipped entirely during import (immutable or structural).
const SKIP_COLUMNS: &[&str] = &["id", "created_at"];

/// Import all project data from the `.gest/` directory into the database.
///
/// For each JSON file, checks file mtime against the row's `updated_at`.
/// If the file is newer, its contents are imported into the database.
/// Deletions are not supported via filesystem — missing files are re-exported.
///
/// Individual file failures are logged as warnings rather than aborting the entire import.
pub async fn import_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  if let Err(e) = import_tasks(conn, project_id, gest_dir).await {
    log::warn!("skipping tasks import: {e}");
  }
  if let Err(e) = import_artifacts(conn, project_id, gest_dir).await {
    log::warn!("skipping artifacts import: {e}");
  }
  if let Err(e) = import_iterations(conn, project_id, gest_dir).await {
    log::warn!("skipping iterations import: {e}");
  }
  if let Err(e) = import_notes(conn, project_id, gest_dir, "task_notes.json", "tasks").await {
    log::warn!("skipping task notes import: {e}");
  }
  if let Err(e) = import_notes(conn, project_id, gest_dir, "artifact_notes.json", "artifacts").await {
    log::warn!("skipping artifact notes import: {e}");
  }
  if let Err(e) = import_events(conn, project_id, gest_dir).await {
    log::warn!("skipping events import: {e}");
  }

  Ok(())
}

async fn import_artifacts(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let path = gest_dir.join("artifacts.json");
  if !path.exists() {
    return Ok(());
  }

  let file_mtime = std::fs::metadata(&path)?.modified()?;
  let content = std::fs::read_to_string(&path)?;
  let artifacts: Vec<Value> = serde_json::from_str(&content)?;

  for artifact_val in artifacts {
    let Some(id_str) = artifact_val.get("id").and_then(|v| v.as_str()) else {
      continue;
    };

    let mut rows = conn
      .query("SELECT updated_at FROM artifacts WHERE id = ?1", [id_str.to_string()])
      .await?;

    if let Some(row) = rows.next().await? {
      let db_updated: String = row.get(0)?;
      if let Ok(db_time) = chrono::DateTime::parse_from_rfc3339(&db_updated) {
        let db_system_time = SystemTime::from(db_time);
        if db_system_time >= file_mtime {
          continue;
        }
      }
    }

    upsert_entity(conn, "artifacts", project_id, &artifact_val).await?;
  }

  // Import artifact bodies from markdown files
  import_artifact_bodies(conn, project_id, gest_dir).await?;

  Ok(())
}

/// Import artifact body content from markdown files in the `artifacts/` subdirectory.
///
/// Prefers `artifacts/index.json` to map filenames back to artifact IDs. When no index
/// exists, falls back to scanning `.md` files and matching by sanitized title for
/// compatibility with older exports.
async fn import_artifact_bodies(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let artifacts_dir = gest_dir.join("artifacts");
  if !artifacts_dir.is_dir() {
    return Ok(());
  }

  let index_path = artifacts_dir.join("index.json");
  if index_path.exists() {
    import_artifact_bodies_via_index(conn, project_id, &artifacts_dir, &index_path).await
  } else {
    import_artifact_bodies_via_title(conn, project_id, &artifacts_dir).await
  }
}

/// Import bodies using `index.json` to resolve filenames to artifact IDs.
async fn import_artifact_bodies_via_index(
  conn: &Connection,
  project_id: &Id,
  artifacts_dir: &Path,
  index_path: &Path,
) -> Result<(), Error> {
  let index_content = std::fs::read_to_string(index_path)?;
  let index: HashMap<String, String> = serde_json::from_str(&index_content)?;

  // Build a reverse map: id_short -> full_id by querying artifacts whose ID starts with the prefix
  for filename in index.keys() {
    let md_path = artifacts_dir.join(filename);
    if !md_path.exists() {
      log::warn!("index references missing file: {filename}");
      continue;
    }

    let id_short = filename.trim_end_matches(".md");
    let body = std::fs::read_to_string(&md_path)?;

    // Find the artifact by ID prefix match within this project
    let mut rows = conn
      .query(
        "SELECT id FROM artifacts WHERE project_id = ?1 AND id LIKE ?2",
        [project_id.to_string(), format!("{id_short}%")],
      )
      .await?;

    if let Some(row) = rows.next().await? {
      let full_id: String = row.get(0)?;
      conn
        .execute(
          "UPDATE artifacts SET body = ?1, updated_at = ?2 WHERE id = ?3",
          [body, Utc::now().to_rfc3339(), full_id],
        )
        .await?;
    } else {
      log::warn!("no artifact found for index entry: {filename}");
    }
  }

  Ok(())
}

/// Fallback: import bodies by matching sanitized title against the filename stem.
async fn import_artifact_bodies_via_title(
  conn: &Connection,
  project_id: &Id,
  artifacts_dir: &Path,
) -> Result<(), Error> {
  for entry in std::fs::read_dir(artifacts_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension().is_some_and(|e| e == "md") {
      let body = match std::fs::read_to_string(&path) {
        Ok(b) => b,
        Err(e) => {
          log::warn!("skipping unreadable artifact file {}: {e}", path.display());
          continue;
        }
      };
      let filename = path.file_stem().unwrap_or_default().to_string_lossy();

      // Find the artifact by title match (sanitized) and update body
      if let Err(e) = conn
        .execute(
          "UPDATE artifacts SET body = ?1, updated_at = ?2 \
            WHERE project_id = ?3 AND LOWER(REPLACE(REPLACE(title, ' ', '-'), '/', '-')) = ?4",
          [
            body,
            Utc::now().to_rfc3339(),
            project_id.to_string(),
            filename.to_lowercase(),
          ],
        )
        .await
      {
        log::warn!("failed to import artifact body from {}: {e}", path.display());
      }
    }
  }

  Ok(())
}

async fn import_events(conn: &Connection, _project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let path = gest_dir.join("events.json");
  if !path.exists() {
    return Ok(());
  }

  let content = std::fs::read_to_string(&path)?;
  let events: Vec<Value> = serde_json::from_str(&content)?;

  for event_val in events {
    let Some(id_str) = event_val.get("id").and_then(|v| v.as_str()) else {
      continue;
    };

    // Only insert if the event doesn't already exist (events are immutable)
    let mut rows = conn
      .query("SELECT 1 FROM events WHERE id = ?1", [id_str.to_string()])
      .await?;

    if rows.next().await?.is_some() {
      continue;
    }

    if let Err(e) = insert_event(conn, &event_val).await {
      log::warn!("skipping malformed event {id_str}: {e}");
    }
  }

  Ok(())
}

async fn import_iterations(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let path = gest_dir.join("iterations.json");
  if !path.exists() {
    return Ok(());
  }

  let file_mtime = std::fs::metadata(&path)?.modified()?;
  let content = std::fs::read_to_string(&path)?;
  let iterations: Vec<Value> = serde_json::from_str(&content)?;

  for iter_val in iterations {
    let Some(id_str) = iter_val.get("id").and_then(|v| v.as_str()) else {
      continue;
    };

    let mut rows = conn
      .query("SELECT updated_at FROM iterations WHERE id = ?1", [id_str.to_string()])
      .await?;

    if let Some(row) = rows.next().await? {
      let db_updated: String = row.get(0)?;
      if let Ok(db_time) = chrono::DateTime::parse_from_rfc3339(&db_updated) {
        let db_system_time = SystemTime::from(db_time);
        if db_system_time >= file_mtime {
          continue;
        }
      }
    }

    upsert_entity(conn, "iterations", project_id, &iter_val).await?;
  }

  Ok(())
}

async fn import_notes(
  conn: &Connection,
  _project_id: &Id,
  gest_dir: &Path,
  filename: &str,
  _entity_table: &str,
) -> Result<(), Error> {
  let path = gest_dir.join(filename);
  if !path.exists() {
    return Ok(());
  }

  let file_mtime = std::fs::metadata(&path)?.modified()?;
  let content = std::fs::read_to_string(&path)?;
  let notes: Vec<Value> = serde_json::from_str(&content)?;

  for note_val in notes {
    let Some(id_str) = note_val.get("id").and_then(|v| v.as_str()) else {
      continue;
    };

    let mut rows = conn
      .query("SELECT updated_at FROM notes WHERE id = ?1", [id_str.to_string()])
      .await?;

    if let Some(row) = rows.next().await? {
      let db_updated: String = row.get(0)?;
      if let Ok(db_time) = chrono::DateTime::parse_from_rfc3339(&db_updated) {
        let db_system_time = SystemTime::from(db_time);
        if db_system_time >= file_mtime {
          continue;
        }
      }
    }

    if let Err(e) = upsert_note(conn, &note_val).await {
      log::warn!("skipping malformed note {id_str}: {e}");
    }
  }

  Ok(())
}

async fn import_tasks(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let path = gest_dir.join("tasks.json");
  if !path.exists() {
    return Ok(());
  }

  let file_mtime = std::fs::metadata(&path)?.modified()?;
  let content = std::fs::read_to_string(&path)?;
  let tasks: Vec<Value> = serde_json::from_str(&content)?;

  for task_val in tasks {
    let Some(id_str) = task_val.get("id").and_then(|v| v.as_str()) else {
      continue;
    };

    // Check if the file version is newer than the DB version
    let mut rows = conn
      .query("SELECT updated_at FROM tasks WHERE id = ?1", [id_str.to_string()])
      .await?;

    if let Some(row) = rows.next().await? {
      let db_updated: String = row.get(0)?;
      if let Ok(db_time) = chrono::DateTime::parse_from_rfc3339(&db_updated) {
        let db_system_time = SystemTime::from(db_time);
        if db_system_time >= file_mtime {
          continue; // DB is newer or equal, skip
        }
      }
    }

    // Upsert the task from the JSON value
    upsert_entity(conn, "tasks", project_id, &task_val).await?;
  }

  Ok(())
}

/// Insert an event from a JSON value. Events are immutable, so this is insert-only.
async fn insert_event(conn: &Connection, value: &Value) -> Result<(), Error> {
  let obj = value
    .as_object()
    .ok_or_else(|| Error::Io(std::io::Error::other("event is not an object")))?;

  let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or_default();
  let entity_id = obj.get("entity_id").and_then(|v| v.as_str()).unwrap_or_default();
  let entity_type = obj.get("entity_type").and_then(|v| v.as_str()).unwrap_or_default();
  let author_id = obj.get("author_id").and_then(|v| v.as_str()).map(|s| s.to_string());
  let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
  let data = obj
    .get("data")
    .map(|v| v.to_string())
    .unwrap_or_else(|| "{}".to_string());
  let description = obj.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
  let event_type = obj.get("event_type").and_then(|v| v.as_str()).unwrap_or_default();

  conn
    .execute(
      "INSERT OR IGNORE INTO events (id, entity_id, entity_type, author_id, created_at, data, description, event_type) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
      libsql::params![
        id.to_string(),
        entity_id.to_string(),
        entity_type.to_string(),
        author_id,
        created_at.to_string(),
        data,
        description,
        event_type.to_string(),
      ],
    )
    .await?;

  Ok(())
}

/// Upsert an entity from a JSON value. Only updates safe, non-FK fields.
async fn upsert_entity(conn: &Connection, table: &str, _project_id: &Id, value: &Value) -> Result<(), Error> {
  let Some(obj) = value.as_object() else {
    return Ok(());
  };

  let Some(id) = obj.get("id").and_then(|v| v.as_str()) else {
    return Ok(());
  };

  let mut sets = Vec::new();
  let mut params: Vec<DbValue> = Vec::new();
  let mut idx = 1;

  for (key, val) in obj {
    if SKIP_COLUMNS.contains(&key.as_str()) || FK_COLUMNS.contains(&key.as_str()) {
      continue;
    }
    sets.push(format!("{key} = ?{idx}"));
    params.push(match val {
      Value::Null => DbValue::Null,
      Value::String(s) => DbValue::from(s.clone()),
      Value::Number(n) => {
        if let Some(i) = n.as_i64() {
          DbValue::from(i)
        } else {
          DbValue::from(n.to_string())
        }
      }
      other => DbValue::from(other.to_string()),
    });
    idx += 1;
  }

  if sets.is_empty() {
    return Ok(());
  }

  let set_clause = sets.join(", ");
  params.push(DbValue::from(id.to_string()));
  let sql = format!("UPDATE {table} SET {set_clause} WHERE id = ?{idx}");

  conn.execute(&sql, libsql::params_from_iter(params)).await?;

  Ok(())
}

/// Upsert a note from a JSON value.
async fn upsert_note(conn: &Connection, value: &Value) -> Result<(), Error> {
  let obj = value
    .as_object()
    .ok_or_else(|| Error::Io(std::io::Error::other("note is not an object")))?;

  let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or_default();
  let entity_id = obj.get("entity_id").and_then(|v| v.as_str()).unwrap_or_default();
  let entity_type = obj.get("entity_type").and_then(|v| v.as_str()).unwrap_or_default();
  let author_id = obj.get("author_id").and_then(|v| v.as_str()).map(|s| s.to_string());
  let body = obj.get("body").and_then(|v| v.as_str()).unwrap_or_default();
  let created_at = obj.get("created_at").and_then(|v| v.as_str()).unwrap_or_default();
  let updated_at = obj.get("updated_at").and_then(|v| v.as_str()).unwrap_or_default();

  conn
    .execute(
      "INSERT INTO notes (id, entity_id, entity_type, author_id, body, created_at, updated_at) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
        ON CONFLICT(id) DO UPDATE SET body = ?5, updated_at = ?7",
      libsql::params![
        id.to_string(),
        entity_id.to_string(),
        entity_type.to_string(),
        author_id,
        body.to_string(),
        created_at.to_string(),
        updated_at.to_string(),
      ],
    )
    .await?;

  Ok(())
}
