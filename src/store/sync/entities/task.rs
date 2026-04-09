//! Per-entity sync adapter for tasks and their notes.
//!
//! - Task definitions live at `task/<id>.yaml` with their tags embedded as a
//!   `tags: […]` field at the top level (per ADR-0016 §3, "Embedding vs.
//!   per-edge files").
//! - Task notes are individual files at `task/notes/<note_id>.yaml`. Each
//!   note carries an `entity_id` referencing its parent task.

use std::{collections::HashSet, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::store::{
  Error,
  model::primitives::{EntityType, Id},
  sync::{paths, yaml},
};

/// On-disk wrapper for `.gest/task/<id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TaskFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  title: String,
  status: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  priority: Option<u8>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  assigned_to: Option<Id>,
  #[serde(default, skip_serializing_if = "String::is_empty")]
  description: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  tags: Vec<String>,
  #[serde(default, skip_serializing_if = "JsonValue::is_null")]
  metadata: JsonValue,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  resolved_at: Option<DateTime<Utc>>,
}

/// On-disk wrapper for `.gest/task/notes/<note_id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TaskNoteFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  entity_id: Id,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  author_id: Option<Id>,
  body: String,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

/// Import every `task/*.yaml` (and `task/notes/*.yaml`) file into SQLite.
pub async fn read_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let task_dir = gest_dir.join(paths::TASK_DIR);
  let notes_dir = task_dir.join(paths::NOTES_DIR);

  for path in yaml::walk_files(&task_dir, "yaml")? {
    if path.starts_with(&notes_dir) {
      continue;
    }
    let Some(file): Option<TaskFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for task {}", file.id.short());
      conn
        .execute("DELETE FROM tasks WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    upsert_task(conn, project_id, &file).await?;
    sync_task_tags(conn, &file).await?;
  }

  for path in yaml::walk_files(&notes_dir, "yaml")? {
    let Some(file): Option<TaskNoteFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for task note {}", file.id.short());
      conn
        .execute("DELETE FROM notes WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    upsert_note(conn, &file).await?;
  }

  Ok(())
}

/// Export every task and task note row to per-entity files.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive_tasks: HashSet<String> = HashSet::new();
  let mut alive_notes: HashSet<String> = HashSet::new();

  // Tasks
  let mut rows = conn
    .query(
      "SELECT id, assigned_to, created_at, description, metadata, priority, resolved_at, status, title, updated_at \
        FROM tasks WHERE project_id = ?1 ORDER BY id",
      [project_id.to_string()],
    )
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let assigned_to: Option<String> = row.get(1).ok();
    let created_at: String = row.get(2)?;
    let description: String = row.get(3)?;
    let metadata_str: String = row.get(4).unwrap_or_else(|_| "{}".to_string());
    let priority: Option<i64> = row.get(5).ok();
    let resolved_at: Option<String> = row.get(6).ok();
    let status: String = row.get(7)?;
    let title: String = row.get(8)?;
    let updated_at: String = row.get(9)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let assigned_to: Option<Id> = match assigned_to {
      Some(s) => Some(s.parse().map_err(|e: String| Error::Io(std::io::Error::other(e)))?),
      None => None,
    };
    let created_at = parse_dt(&created_at)?;
    let updated_at = parse_dt(&updated_at)?;
    let resolved_at = resolved_at.as_deref().map(parse_dt).transpose()?;
    let metadata: JsonValue = serde_json::from_str(&metadata_str).unwrap_or(JsonValue::Null);

    let tags = load_task_tag_labels(conn, &id).await?;

    let file = TaskFile {
      deleted_at: None,
      id: id.clone(),
      title,
      status,
      priority: priority.and_then(|p| u8::try_from(p).ok()),
      assigned_to,
      description,
      tags,
      metadata,
      created_at,
      updated_at,
      resolved_at,
    };
    let path = paths::task_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive_tasks.insert(id.to_string());
  }

  // Task notes
  let mut rows = conn
    .query(
      "SELECT n.id, n.entity_id, n.author_id, n.body, n.created_at, n.updated_at \
        FROM notes n \
        WHERE n.entity_type = 'task' AND n.entity_id IN (SELECT id FROM tasks WHERE project_id = ?1) \
        ORDER BY n.id",
      [project_id.to_string()],
    )
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let entity_id_str: String = row.get(1)?;
    let author_id: Option<String> = row.get(2).ok();
    let body: String = row.get(3)?;
    let created_at: String = row.get(4)?;
    let updated_at: String = row.get(5)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let entity_id: Id = entity_id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let author_id = match author_id {
      Some(s) => Some(s.parse().map_err(|e: String| Error::Io(std::io::Error::other(e)))?),
      None => None,
    };
    let file = TaskNoteFile {
      deleted_at: None,
      id: id.clone(),
      entity_id,
      author_id,
      body,
      created_at: parse_dt(&created_at)?,
      updated_at: parse_dt(&updated_at)?,
    };
    let path = paths::task_note_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive_notes.insert(id.to_string());
  }

  // Clean up files for tasks/notes that no longer exist in SQLite. The notes
  // directory is a child of the task directory, so prune notes first to avoid
  // walking newly-deleted files when pruning tasks.
  let task_dir = gest_dir.join(paths::TASK_DIR);
  let notes_dir = task_dir.join(paths::NOTES_DIR);
  yaml::cleanup_orphans(conn, project_id, gest_dir, &notes_dir, "yaml", &alive_notes).await?;
  // Walk only direct children of task_dir, skipping the notes subdirectory.
  for entry in std::fs::read_dir(&task_dir).into_iter().flatten().flatten() {
    let path = entry.path();
    if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
      let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
      // Tombstoned files are intentional delete markers; leave them in
      // place so downstream clones pick up the deletion on their next
      // import.
      if !alive_tasks.contains(stem) && !yaml::is_tombstoned_yaml_file(&path) {
        let relative = paths::relative(gest_dir, &path).unwrap_or_default();
        std::fs::remove_file(&path)?;
        conn
          .execute(
            "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
            [relative, project_id.to_string()],
          )
          .await?;
      }
    }
  }

  Ok(())
}

async fn load_task_tag_labels(conn: &Connection, task_id: &Id) -> Result<Vec<String>, Error> {
  let mut rows = conn
    .query(
      "SELECT t.label FROM entity_tags et JOIN tags t ON t.id = et.tag_id \
        WHERE et.entity_type = 'task' AND et.entity_id = ?1 ORDER BY t.label",
      [task_id.to_string()],
    )
    .await?;
  let mut labels = Vec::new();
  while let Some(row) = rows.next().await? {
    labels.push(row.get::<String>(0)?);
  }
  Ok(labels)
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>, Error> {
  DateTime::parse_from_rfc3339(s)
    .map(|dt| dt.with_timezone(&Utc))
    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}

async fn sync_task_tags(conn: &Connection, file: &TaskFile) -> Result<(), Error> {
  // Detach all current tag attachments for this task, then re-attach the
  // labels listed in the file. Tag definitions must already exist (created
  // by the `tag` adapter, which runs first in the orchestrator order).
  conn
    .execute(
      "DELETE FROM entity_tags WHERE entity_type = 'task' AND entity_id = ?1",
      [file.id.to_string()],
    )
    .await?;
  for label in &file.tags {
    let mut rows = conn
      .query("SELECT id FROM tags WHERE label = ?1", [label.clone()])
      .await?;
    let tag_id: String = match rows.next().await? {
      Some(row) => row.get(0)?,
      None => {
        // Auto-create the tag definition if missing.
        let new_id = Id::new();
        conn
          .execute(
            "INSERT INTO tags (id, label) VALUES (?1, ?2)",
            [new_id.to_string(), label.clone()],
          )
          .await?;
        new_id.to_string()
      }
    };
    conn
      .execute(
        "INSERT OR IGNORE INTO entity_tags (entity_type, entity_id, tag_id) VALUES ('task', ?1, ?2)",
        [file.id.to_string(), tag_id],
      )
      .await?;
  }
  Ok(())
}

async fn upsert_note(conn: &Connection, file: &TaskNoteFile) -> Result<(), Error> {
  conn
    .execute(
      "INSERT INTO notes (id, entity_id, entity_type, author_id, body, created_at, updated_at) \
        VALUES (?1, ?2, 'task', ?3, ?4, ?5, ?6) \
        ON CONFLICT(id) DO UPDATE SET body = ?4, updated_at = ?6",
      libsql::params![
        file.id.to_string(),
        file.entity_id.to_string(),
        file.author_id.as_ref().map(|i| i.to_string()),
        file.body.clone(),
        file.created_at.to_rfc3339(),
        file.updated_at.to_rfc3339(),
      ],
    )
    .await?;
  Ok(())
}

async fn upsert_task(conn: &Connection, project_id: &Id, file: &TaskFile) -> Result<(), Error> {
  let metadata_str = serde_json::to_string(&file.metadata).unwrap_or_else(|_| "{}".to_string());
  conn
    .execute(
      "INSERT INTO tasks (id, project_id, assigned_to, created_at, description, metadata, priority, \
        resolved_at, status, title, updated_at) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
        ON CONFLICT(id) DO UPDATE SET assigned_to = ?3, description = ?5, metadata = ?6, \
        priority = ?7, resolved_at = ?8, status = ?9, title = ?10, updated_at = ?11",
      libsql::params![
        file.id.to_string(),
        project_id.to_string(),
        file.assigned_to.as_ref().map(|i| i.to_string()),
        file.created_at.to_rfc3339(),
        file.description.clone(),
        metadata_str,
        file.priority.map(|p| p as i64),
        file.resolved_at.map(|d| d.to_rfc3339()),
        file.status.clone(),
        file.title.clone(),
        file.updated_at.to_rfc3339(),
      ],
    )
    .await?;
  let _ = EntityType::Task; // ensure import is exercised in case of future use
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use tempfile::TempDir;

  use super::*;
  use crate::store;

  async fn setup() -> (std::sync::Arc<store::Db>, TempDir, Id, PathBuf) {
    let (db, _tmp_db) = store::open_temp().await.unwrap();
    let conn = db.connect().await.unwrap();
    let project_root = TempDir::new().unwrap();
    let gest_dir = project_root.path().join(".gest");
    std::fs::create_dir_all(&gest_dir).unwrap();
    let pid = Id::new();
    conn
      .execute(
        "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
          pid.to_string(),
          project_root.path().to_string_lossy().into_owned(),
          "2026-04-08T00:00:00Z".to_string(),
          "2026-04-08T00:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    std::mem::forget(_tmp_db);
    (db, project_root, pid, gest_dir)
  }

  async fn insert_task(conn: &libsql::Connection, project_id: &Id, title: &str) -> Id {
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title, status, created_at, updated_at) \
          VALUES (?1, ?2, ?3, 'open', ?4, ?4)",
        libsql::params![
          id.to_string(),
          project_id.to_string(),
          title.to_string(),
          "2026-04-08T00:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    id
  }

  async fn attach_tag(conn: &libsql::Connection, task_id: &Id, label: &str) {
    let tag_id = Id::new();
    conn
      .execute(
        "INSERT INTO tags (id, label) VALUES (?1, ?2)",
        [tag_id.to_string(), label.to_string()],
      )
      .await
      .unwrap();
    conn
      .execute(
        "INSERT INTO entity_tags (entity_type, entity_id, tag_id) VALUES ('task', ?1, ?2)",
        [task_id.to_string(), tag_id.to_string()],
      )
      .await
      .unwrap();
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_hard_deletes_for_a_tombstoned_task_file() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let task_id = insert_task(&conn, &pid, "to delete").await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::task_path(&gest_dir, &task_id);
      let mut content = std::fs::read_to_string(&path).unwrap();
      content.insert_str(0, "deleted_at: 2026-04-08T12:00:00Z\n");
      std::fs::write(&path, content).unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM tasks WHERE id = ?1", [task_id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_roundtrips_task_and_tags_through_disk() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let task_id = insert_task(&conn, &pid, "roundtrip").await;
      attach_tag(&conn, &task_id, "x").await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      // Wipe SQLite state for tasks/tags.
      conn.execute("DELETE FROM entity_tags", ()).await.unwrap();
      conn.execute("DELETE FROM tasks", ()).await.unwrap();
      conn.execute("DELETE FROM tags", ()).await.unwrap();

      // Tags must exist before tasks (orchestrator handles this in production).
      let tag_id = Id::new();
      conn
        .execute(
          "INSERT INTO tags (id, label) VALUES (?1, ?2)",
          [tag_id.to_string(), "x".to_string()],
        )
        .await
        .unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT title FROM tasks WHERE id = ?1", [task_id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let title: String = row.get(0).unwrap();
      assert_eq!(title, "roundtrip");

      let mut rows = conn
        .query(
          "SELECT t.label FROM entity_tags et JOIN tags t ON t.id = et.tag_id WHERE et.entity_id = ?1",
          [task_id.to_string()],
        )
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let label: String = row.get(0).unwrap();
      assert_eq!(label, "x");
    }
  }

  mod write_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_writes_a_task_yaml_file_with_embedded_tags() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let task_id = insert_task(&conn, &pid, "demo task").await;
      attach_tag(&conn, &task_id, "bug").await;
      attach_tag(&conn, &task_id, "p0").await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::task_path(&gest_dir, &task_id);
      assert!(path.exists());
      let raw = std::fs::read_to_string(&path).unwrap();
      assert!(raw.contains("title: demo task"));
      assert!(raw.contains("tags:"));
      assert!(raw.contains("- bug"));
      assert!(raw.contains("- p0"));
    }

    #[tokio::test]
    async fn it_writes_each_task_note_to_its_own_file() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let task_id = insert_task(&conn, &pid, "with notes").await;
      let note_id = Id::new();
      conn
        .execute(
          "INSERT INTO notes (id, entity_id, entity_type, body, created_at, updated_at) \
            VALUES (?1, ?2, 'task', ?3, ?4, ?4)",
          libsql::params![
            note_id.to_string(),
            task_id.to_string(),
            "first note".to_string(),
            "2026-04-08T00:00:00Z".to_string(),
          ],
        )
        .await
        .unwrap();

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let note_path = paths::task_note_path(&gest_dir, &note_id);
      assert!(note_path.exists());
      let raw = std::fs::read_to_string(&note_path).unwrap();
      assert!(raw.contains("body: first note"));
      assert_eq!(raw.contains(&format!("entity_id: {}", task_id)), true);
    }
  }
}
