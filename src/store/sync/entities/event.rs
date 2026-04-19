//! Per-entity sync adapter for events.
//!
//! Events are append-only and live at `event/<yyyy-mm>/<event_id>.yaml`.
//! Monthly sharding (per ADR-0016 §4) keeps any single directory manageable.
//! Two collaborators recording new events on the same day write different
//! files, so events never produce merge conflicts.
//!
//! ## Data source
//!
//! Events live in the `transaction_events` table. ADR-0016 §8 says
//! transactions and their `before_data` snapshots are local-only undo state
//! that should NOT be synced; this adapter therefore only writes events that
//! carry a `semantic_type` (the audit-log subset) and only the audit-log
//! columns. The `transaction_id` and `before_data` columns stay local.

use std::{collections::HashSet, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{
  Error,
  model::primitives::Id,
  sync::{paths, yaml},
};

/// Insert SQL for importing a synced event into `transaction_events`.
const INSERT_TRANSACTION_EVENT_SQL: &str = "INSERT INTO transaction_events \
  (id, transaction_id, before_data, created_at, event_type, row_id, table_name, semantic_type, old_value, new_value) \
  VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
  ON CONFLICT(id) DO NOTHING";

/// Select SQL for the audit-log subset of `transaction_events` scoped to a project.
const SELECT_TRANSACTION_EVENTS_SQL: &str = "SELECT \
  te.id, te.table_name, te.row_id, te.event_type, te.semantic_type, te.old_value, te.new_value, te.created_at \
  FROM transaction_events te \
  JOIN transactions t ON t.id = te.transaction_id \
  WHERE t.project_id = ?1 AND te.semantic_type IS NOT NULL \
  ORDER BY te.created_at, te.id";

/// On-disk wrapper for `.gest/event/<yyyy-mm>/<id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct EventFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  table_name: String,
  row_id: String,
  event_type: String,
  semantic_type: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  old_value: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  new_value: Option<String>,
  created_at: DateTime<Utc>,
}

/// Import every `event/**/*.yaml` file into SQLite.
///
/// Imported events are inserted directly into `transaction_events` with no
/// owning transaction (the source collaborator's transaction is local). The
/// `transaction_id` column is NOT NULL, so this adapter creates a synthetic
/// "synced" transaction per import batch to satisfy the foreign key.
pub async fn read_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let dir = gest_dir.join(paths::EVENT_DIR);
  let files = yaml::walk_files(&dir, "yaml")?;
  if files.is_empty() {
    return Ok(());
  }

  // Create one synthetic transaction to anchor all imported events. Mark it
  // already-undone so it does not appear in the "latest undoable transaction"
  // query — collaborators' synced events are not part of the local user's
  // undo history.
  let synthetic_tx = Id::new();
  let now = Utc::now().to_rfc3339();
  conn
    .execute(
      "INSERT INTO transactions (id, project_id, command, created_at, undone_at) \
        VALUES (?1, ?2, 'sync import', ?3, ?3)",
      [synthetic_tx.to_string(), project_id.to_string(), now],
    )
    .await?;

  let mut imported = 0u64;
  for path in files {
    let Some(file): Option<EventFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for event {}", file.id.short());
      conn
        .execute("DELETE FROM transaction_events WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    conn
      .execute(
        INSERT_TRANSACTION_EVENT_SQL,
        libsql::params![
          file.id.to_string(),
          synthetic_tx.to_string(),
          file.created_at.to_rfc3339(),
          file.event_type.clone(),
          file.row_id.clone(),
          file.table_name.clone(),
          file.semantic_type.clone(),
          file.old_value.clone(),
          file.new_value.clone(),
        ],
      )
      .await?;
    imported += 1;
  }

  // If no events were actually imported, drop the synthetic transaction so it
  // doesn't pollute the local undo history.
  if imported == 0 {
    conn
      .execute("DELETE FROM transactions WHERE id = ?1", [synthetic_tx.to_string()])
      .await?;
  }
  Ok(())
}

/// Export the audit-log subset of `transaction_events` to per-event YAML files.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive: HashSet<String> = HashSet::new();
  let mut rows = conn
    .query(SELECT_TRANSACTION_EVENTS_SQL, [project_id.to_string()])
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let table_name: String = row.get(1)?;
    let row_id: String = row.get(2)?;
    let event_type: String = row.get(3)?;
    let semantic_type: String = row.get(4)?;
    let old_value: Option<String> = row.get(5).ok();
    let new_value: Option<String> = row.get(6).ok();
    let created_at_str: String = row.get(7)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
      .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
      .with_timezone(&Utc);

    let file = EventFile {
      deleted_at: None,
      id: id.clone(),
      table_name,
      row_id,
      event_type,
      semantic_type,
      old_value,
      new_value,
      created_at,
    };
    let path = paths::event_path(gest_dir, &id, &created_at);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive.insert(id.to_string());
  }

  let dir = gest_dir.join(paths::EVENT_DIR);
  yaml::cleanup_orphans(conn, project_id, gest_dir, &dir, "yaml", &alive).await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use chrono::TimeZone;
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

  async fn insert_semantic_event(conn: &libsql::Connection, project_id: &Id) -> (Id, DateTime<Utc>) {
    let tx_id = Id::new();
    conn
      .execute(
        "INSERT INTO transactions (id, project_id, command, created_at) VALUES (?1, ?2, 'test', ?3)",
        [
          tx_id.to_string(),
          project_id.to_string(),
          "2026-04-08T12:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    let event_id = Id::new();
    let when = Utc.with_ymd_and_hms(2026, 4, 8, 12, 0, 0).unwrap();
    conn
      .execute(
        "INSERT INTO transaction_events \
          (id, transaction_id, created_at, event_type, row_id, table_name, semantic_type, old_value, new_value) \
          VALUES (?1, ?2, ?3, 'modified', 'kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk', \
          'tasks', 'status-change', 'open', 'in-progress')",
        libsql::params![event_id.to_string(), tx_id.to_string(), when.to_rfc3339()],
      )
      .await
      .unwrap();
    (event_id, when)
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_roundtrips_an_event_through_disk() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let (event_id, _when) = insert_semantic_event(&conn, &pid).await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      conn.execute("DELETE FROM transaction_events", ()).await.unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query(
          "SELECT semantic_type FROM transaction_events WHERE id = ?1",
          [event_id.to_string()],
        )
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let semantic_type: String = row.get(0).unwrap();
      assert_eq!(semantic_type, "status-change");
    }
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_shards_event_files_under_yyyy_mm_directories() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let (event_id, when) = insert_semantic_event(&conn, &pid).await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::event_path(&gest_dir, &event_id, &when);
      assert!(path.exists());
      assert!(path.to_string_lossy().contains("/event/2026-04/"));
    }

    #[tokio::test]
    async fn it_skips_events_without_a_semantic_type() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let tx_id = Id::new();
      conn
        .execute(
          "INSERT INTO transactions (id, project_id, command, created_at) VALUES (?1, ?2, 't', ?3)",
          [tx_id.to_string(), pid.to_string(), "2026-04-08T00:00:00Z".to_string()],
        )
        .await
        .unwrap();
      let raw_event_id = Id::new();
      conn
        .execute(
          "INSERT INTO transaction_events (id, transaction_id, created_at, event_type, row_id, table_name) \
            VALUES (?1, ?2, ?3, 'modified', 'k', 'tasks')",
          [
            raw_event_id.to_string(),
            tx_id.to_string(),
            "2026-04-08T00:00:00Z".to_string(),
          ],
        )
        .await
        .unwrap();

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let event_dir = gest_dir.join("event");
      assert!(!event_dir.exists() || std::fs::read_dir(&event_dir).unwrap().count() == 0);
    }
  }
}
