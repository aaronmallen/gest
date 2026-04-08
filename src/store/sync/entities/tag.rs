//! Per-entity sync adapter for tag definitions.
//!
//! This adapter handles the tag DEFINITION rows from the `tags` table — one
//! file per tag at `tag/<id>.yaml`. Tag attachments (the `entity_tags` join
//! rows) are embedded in their parent entity files per ADR-0016, not synced
//! through this module.

use std::{collections::HashSet, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{
  model::primitives::Id,
  sync::{Error, paths, yaml},
};

/// On-disk wrapper for `.gest/tag/<id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TagFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  label: String,
}

/// Import every `tag/*.yaml` file under `gest_dir` into SQLite.
pub async fn read_all(conn: &Connection, _project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let dir = gest_dir.join(paths::TAG_DIR);
  for path in yaml::walk_files(&dir, "yaml")? {
    let Some(file): Option<TagFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for tag {}", file.id.short());
      conn
        .execute("DELETE FROM tags WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    conn
      .execute(
        "INSERT INTO tags (id, label) VALUES (?1, ?2) \
          ON CONFLICT(id) DO UPDATE SET label = ?2",
        [file.id.to_string(), file.label.clone()],
      )
      .await?;
  }
  Ok(())
}

/// Export every tag row to `tag/<id>.yaml`.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive: HashSet<String> = HashSet::new();
  let mut rows = conn.query("SELECT id, label FROM tags ORDER BY id", ()).await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let label: String = row.get(1)?;
    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;

    let file = TagFile {
      deleted_at: None,
      id: id.clone(),
      label,
    };
    let path = paths::tag_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive.insert(id.to_string());
  }

  let dir = gest_dir.join(paths::TAG_DIR);
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

  async fn insert_tag(conn: &libsql::Connection, label: &str) -> Id {
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO tags (id, label) VALUES (?1, ?2)",
        [id.to_string(), label.to_string()],
      )
      .await
      .unwrap();
    id
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_writes_one_yaml_file_per_tag() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id_a = insert_tag(&conn, "bug").await;
      let id_b = insert_tag(&conn, "feature").await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      assert!(paths::tag_path(&gest_dir, &id_a).exists());
      assert!(paths::tag_path(&gest_dir, &id_b).exists());
      let raw = std::fs::read_to_string(paths::tag_path(&gest_dir, &id_a)).unwrap();
      assert!(raw.contains("label: bug"));
    }
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_roundtrips_tags_through_disk() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_tag(&conn, "design").await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      conn
        .execute("DELETE FROM tags WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT label FROM tags WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let label: String = row.get(0).unwrap();
      assert_eq!(label, "design");
    }

    #[tokio::test]
    async fn it_hard_deletes_for_a_tombstoned_file() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_tag(&conn, "stale").await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      let path = paths::tag_path(&gest_dir, &id);
      let mut content = std::fs::read_to_string(&path).unwrap();
      content.insert_str(0, "deleted_at: 2026-04-08T12:00:00Z\n");
      std::fs::write(&path, content).unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM tags WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }
  }
}
