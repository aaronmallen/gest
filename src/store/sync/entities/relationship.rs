//! Per-entity sync adapter for relationships.
//!
//! Relationships are symmetric edges between two entities. Each row gets its
//! own file at `relationship/<id>.yaml` so that adding or removing a
//! relationship touches no other file (per ADR-0016 §3 — "per-edge files").

use std::{collections::HashSet, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{
  model::primitives::Id,
  sync::{Error, paths, yaml},
};

/// On-disk wrapper for `.gest/relationship/<id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct RelationshipFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  rel_type: String,
  source_type: String,
  source_id: Id,
  target_type: String,
  target_id: Id,
}

/// Import every `relationship/*.yaml` file into SQLite.
pub async fn read_all(conn: &Connection, _project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let dir = gest_dir.join(paths::RELATIONSHIP_DIR);
  for path in yaml::walk_files(&dir, "yaml")? {
    let Some(file): Option<RelationshipFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for relationship {}", file.id.short());
      conn
        .execute("DELETE FROM relationships WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    conn
      .execute(
        "INSERT INTO relationships (id, rel_type, source_id, source_type, target_id, target_type) \
          VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
          ON CONFLICT(id) DO UPDATE SET rel_type = ?2, source_id = ?3, source_type = ?4, \
          target_id = ?5, target_type = ?6",
        [
          file.id.to_string(),
          file.rel_type.clone(),
          file.source_id.to_string(),
          file.source_type.clone(),
          file.target_id.to_string(),
          file.target_type.clone(),
        ],
      )
      .await?;
  }
  Ok(())
}

/// Export every relationship row scoped to this project's entities to disk.
///
/// Relationships have no `project_id` column, so we export every relationship
/// whose source or target is a known entity in this project. Two collaborators
/// adding edges to the same project produce different files and merge cleanly.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive: HashSet<String> = HashSet::new();
  let mut rows = conn
    .query(
      "SELECT DISTINCT r.id, r.rel_type, r.source_id, r.source_type, r.target_id, r.target_type \
        FROM relationships r \
        WHERE r.source_id IN (SELECT id FROM tasks WHERE project_id = ?1) \
          OR r.source_id IN (SELECT id FROM artifacts WHERE project_id = ?1) \
          OR r.source_id IN (SELECT id FROM iterations WHERE project_id = ?1) \
          OR r.target_id IN (SELECT id FROM tasks WHERE project_id = ?1) \
          OR r.target_id IN (SELECT id FROM artifacts WHERE project_id = ?1) \
          OR r.target_id IN (SELECT id FROM iterations WHERE project_id = ?1) \
        ORDER BY r.id",
      [project_id.to_string()],
    )
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let rel_type: String = row.get(1)?;
    let source_id_str: String = row.get(2)?;
    let source_type: String = row.get(3)?;
    let target_id_str: String = row.get(4)?;
    let target_type: String = row.get(5)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let source_id: Id = source_id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let target_id: Id = target_id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;

    let file = RelationshipFile {
      deleted_at: None,
      id: id.clone(),
      rel_type,
      source_type,
      source_id,
      target_type,
      target_id,
    };
    let path = paths::relationship_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive.insert(id.to_string());
  }

  let dir = gest_dir.join(paths::RELATIONSHIP_DIR);
  yaml::cleanup_orphans(conn, project_id, gest_dir, &dir, "yaml", &alive).await?;
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

  async fn insert_task(conn: &libsql::Connection, project_id: &Id) -> Id {
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title, status, created_at, updated_at) \
          VALUES (?1, ?2, 't', 'open', ?3, ?3)",
        libsql::params![
          id.to_string(),
          project_id.to_string(),
          "2026-04-08T00:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    id
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_writes_one_yaml_file_per_relationship() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let a = insert_task(&conn, &pid).await;
      let b = insert_task(&conn, &pid).await;
      let rel_id = Id::new();
      conn
        .execute(
          "INSERT INTO relationships (id, rel_type, source_id, source_type, target_id, target_type) \
            VALUES (?1, 'blocks', ?2, 'task', ?3, 'task')",
          [rel_id.to_string(), a.to_string(), b.to_string()],
        )
        .await
        .unwrap();

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::relationship_path(&gest_dir, &rel_id);
      assert!(path.exists());
      let raw = std::fs::read_to_string(&path).unwrap();
      assert!(raw.contains("rel_type: blocks"));
    }
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_roundtrips_a_relationship_through_disk() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let a = insert_task(&conn, &pid).await;
      let b = insert_task(&conn, &pid).await;
      let rel_id = Id::new();
      conn
        .execute(
          "INSERT INTO relationships (id, rel_type, source_id, source_type, target_id, target_type) \
            VALUES (?1, 'blocks', ?2, 'task', ?3, 'task')",
          [rel_id.to_string(), a.to_string(), b.to_string()],
        )
        .await
        .unwrap();
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      conn.execute("DELETE FROM relationships", ()).await.unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT rel_type FROM relationships WHERE id = ?1", [rel_id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let rel_type: String = row.get(0).unwrap();
      assert_eq!(rel_type, "blocks");
    }
  }
}
