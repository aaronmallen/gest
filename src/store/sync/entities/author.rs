//! Per-entity sync adapter for authors.
//!
//! Authors live in the global `authors` table (no `project_id` column), but
//! every project mirrors the full author set into its `.gest/author/` directory
//! so a fresh checkout can resolve every author referenced by tasks, notes,
//! and events.

use std::{collections::HashSet, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{
  model::primitives::{AuthorType, Id},
  sync::{Error, paths, yaml},
};

/// On-disk wrapper for `.gest/author/<id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AuthorFile {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  email: Option<String>,
  author_type: AuthorType,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

/// Import every `author/*.yaml` file under `gest_dir` into SQLite.
pub async fn read_all(conn: &Connection, _project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let dir = gest_dir.join(paths::AUTHOR_DIR);
  for path in yaml::walk_files(&dir, "yaml")? {
    let Some(file): Option<AuthorFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for author {}", file.id.short());
      conn
        .execute("DELETE FROM authors WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    upsert(conn, &file).await?;
  }
  Ok(())
}

/// Export every author row to `author/<id>.yaml`.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive: HashSet<String> = HashSet::new();
  let mut rows = conn
    .query(
      "SELECT id, name, email, author_type, created_at, updated_at FROM authors ORDER BY id",
      (),
    )
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let name: String = row.get(1)?;
    let email: Option<String> = row.get(2).ok();
    let author_type_str: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let updated_at_str: String = row.get(5)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let author_type: AuthorType = author_type_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
      .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
      .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
      .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?
      .with_timezone(&Utc);

    let file = AuthorFile {
      deleted_at: None,
      id: id.clone(),
      name,
      email,
      author_type,
      created_at,
      updated_at,
    };
    let path = paths::author_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive.insert(id.to_string());
  }

  let dir = gest_dir.join(paths::AUTHOR_DIR);
  yaml::cleanup_orphans(conn, project_id, gest_dir, &dir, "yaml", &alive).await?;
  Ok(())
}

async fn upsert(conn: &Connection, file: &AuthorFile) -> Result<(), Error> {
  conn
    .execute(
      "INSERT INTO authors (id, author_type, name, email, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
        ON CONFLICT(id) DO UPDATE SET author_type = ?2, name = ?3, email = ?4, updated_at = ?6",
      libsql::params![
        file.id.to_string(),
        file.author_type.to_string(),
        file.name.clone(),
        file.email.clone(),
        file.created_at.to_rfc3339(),
        file.updated_at.to_rfc3339(),
      ],
    )
    .await?;
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

  async fn insert_author(conn: &libsql::Connection, name: &str, email: Option<&str>) -> Id {
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO authors (id, name, email, author_type, created_at) VALUES (?1, ?2, ?3, 'human', ?4)",
        libsql::params![
          id.to_string(),
          name.to_string(),
          email.map(|s| s.to_string()),
          "2026-04-08T00:00:00Z".to_string(),
        ],
      )
      .await
      .unwrap();
    id
  }

  mod read_all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_hard_deletes_for_a_tombstoned_file() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_author(&conn, "Dave", None).await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::author_path(&gest_dir, &id);
      let mut content = std::fs::read_to_string(&path).unwrap();
      content.insert_str(0, "deleted_at: 2026-04-08T12:00:00Z\n");
      std::fs::write(&path, content).unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM authors WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_roundtrips_authors_through_disk() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_author(&conn, "Carol", Some("carol@example.com")).await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      conn
        .execute("DELETE FROM authors WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT name, email FROM authors WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let name: String = row.get(0).unwrap();
      let email: String = row.get(1).unwrap();
      assert_eq!(name, "Carol");
      assert_eq!(email, "carol@example.com");
    }
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_writes_one_yaml_file_per_author() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id_a = insert_author(&conn, "Alice", Some("alice@example.com")).await;
      let id_b = insert_author(&conn, "Bob", None).await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path_a = paths::author_path(&gest_dir, &id_a);
      let path_b = paths::author_path(&gest_dir, &id_b);
      assert!(path_a.exists());
      assert!(path_b.exists());
      let raw_a = std::fs::read_to_string(&path_a).unwrap();
      assert!(raw_a.contains("name: Alice"));
      assert!(raw_a.contains("email: alice@example.com"));
      let raw_b = std::fs::read_to_string(&path_b).unwrap();
      assert!(raw_b.contains("name: Bob"));
      assert!(!raw_b.contains("email:"));
    }
  }
}
