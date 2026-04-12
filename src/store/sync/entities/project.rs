//! Per-entity sync adapter for the singleton `project.yaml` file.
//!
//! Unlike the other entity adapters, there is exactly one project per
//! `.gest/` directory: it is the project that owns the gest_dir. Only fields
//! that should travel with the repository (id, created_at, updated_at) are
//! synced; the local checkout's `root` path stays out of the file because it
//! varies per collaborator.

use std::path::Path;

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{
  Error,
  model::primitives::Id,
  sync::{paths, yaml},
};

/// On-disk wrapper for `.gest/project.yaml`.
///
/// Field declaration order is the on-disk order. `deleted_at` is first so it
/// is immediately visible in diffs when a project is tombstoned.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ProjectFile {
  /// When the project was soft-archived; absent for active projects.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  archived_at: Option<DateTime<Utc>>,
  /// Tombstone instant; absent for live projects.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  /// Stable project identifier shared across collaborators.
  id: Id,
  /// When this project was first created.
  created_at: DateTime<Utc>,
  /// When this project's metadata was last modified.
  updated_at: DateTime<Utc>,
}

/// Import the project from `.gest/project.yaml` into SQLite.
///
/// If the file is absent, this is a no-op (the project row already exists in
/// SQLite or will be created by another path). If the file carries a tombstone
/// the corresponding project row is hard-deleted.
pub async fn read_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let path = paths::project_path(gest_dir);
  let Some(file): Option<ProjectFile> = yaml::read(&path)? else {
    return Ok(());
  };

  if file.deleted_at.is_some() {
    log::info!("sync import: tombstone for project {}", file.id.short());
    conn
      .execute("DELETE FROM projects WHERE id = ?1", [file.id.to_string()])
      .await?;
    return Ok(());
  }

  // Upsert the project row, preserving the local `root` if the row already
  // exists. New rows take their root from the parent of `gest_dir`.
  let mut existing = conn
    .query("SELECT root FROM projects WHERE id = ?1", [file.id.to_string()])
    .await?;
  let archived_at_str = file.archived_at.map(|dt| dt.to_rfc3339());
  if let Some(row) = existing.next().await? {
    let root: String = row.get(0)?;
    conn
      .execute(
        "UPDATE projects SET archived_at = ?2, created_at = ?3, updated_at = ?4, root = ?5 WHERE id = ?1",
        libsql::params![
          file.id.to_string(),
          archived_at_str,
          file.created_at.to_rfc3339(),
          file.updated_at.to_rfc3339(),
          root,
        ],
      )
      .await?;
  } else {
    let root = gest_dir
      .parent()
      .map(|p| p.to_string_lossy().into_owned())
      .unwrap_or_default();
    conn
      .execute(
        "INSERT INTO projects (id, root, archived_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        libsql::params![
          file.id.to_string(),
          root,
          archived_at_str,
          file.created_at.to_rfc3339(),
          file.updated_at.to_rfc3339(),
        ],
      )
      .await?;
  }
  let _ = project_id;
  Ok(())
}

/// Export the project row to `.gest/project.yaml`.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut rows = conn
    .query(
      "SELECT id, archived_at, created_at, updated_at FROM projects WHERE id = ?1",
      [project_id.to_string()],
    )
    .await?;
  let Some(row) = rows.next().await? else {
    return Ok(());
  };
  let id_str: String = row.get(0)?;
  let archived_at: Option<String> = row.get(1)?;
  let created_at: String = row.get(2)?;
  let updated_at: String = row.get(3)?;
  let id: Id = id_str
    .parse()
    .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
  let archived_at = archived_at
    .map(|s| {
      DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
    })
    .transpose()?;
  let created_at = DateTime::parse_from_rfc3339(&created_at)
    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
    .with_timezone(&Utc);
  let updated_at = DateTime::parse_from_rfc3339(&updated_at)
    .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
    .with_timezone(&Utc);

  let file = ProjectFile {
    archived_at,
    deleted_at: None,
    id,
    created_at,
    updated_at,
  };

  let path = paths::project_path(gest_dir);
  yaml::write_cached(conn, project_id, gest_dir, &path, &file).await
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
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
          id.to_string(),
          project_root.path().to_string_lossy().into_owned(),
          "2026-04-08T00:00:00Z".to_string(),
          "2026-04-08T00:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    std::mem::forget(_tmp_db);
    (db, project_root, id, gest_dir)
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_hard_deletes_the_row_for_a_tombstoned_file() {
      let (db, _root, id, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      write_all(&conn, &id, &gest_dir).await.unwrap();

      let path = gest_dir.join("project.yaml");
      let tombstoned = ProjectFile {
        archived_at: None,
        deleted_at: Some(Utc.with_ymd_and_hms(2026, 4, 8, 12, 0, 0).unwrap()),
        id: id.clone(),
        created_at: Utc.with_ymd_and_hms(2026, 4, 8, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 4, 8, 0, 0, 0).unwrap(),
      };
      std::fs::write(&path, yaml_serde::to_string(&tombstoned).unwrap()).unwrap();

      read_all(&conn, &id, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM projects WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_is_a_noop_when_the_file_is_absent() {
      let (db, _root, id, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();

      read_all(&conn, &id, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM projects WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert_eq!(rows.next().await.unwrap().is_some(), true);
    }

    #[tokio::test]
    async fn it_roundtrips_a_project_through_disk() {
      let (db, _root, id, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();

      write_all(&conn, &id, &gest_dir).await.unwrap();
      conn
        .execute("DELETE FROM projects WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();

      read_all(&conn, &id, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM projects WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_some());
    }
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_writes_project_yaml_with_synced_fields_only() {
      let (db, _root, id, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();

      write_all(&conn, &id, &gest_dir).await.unwrap();

      let path = gest_dir.join("project.yaml");
      assert!(path.exists());
      let raw = std::fs::read_to_string(&path).unwrap();
      assert!(raw.contains(&format!("id: {}", id)));
      assert!(raw.contains("created_at:"));
      assert!(raw.contains("updated_at:"));
      // local-only field must not leak
      assert!(!raw.contains("root:"));
    }
  }
}
