//! Per-entity sync adapter for artifacts and their notes.
//!
//! - Artifact bodies live at `artifact/<id>.md` as markdown with YAML
//!   frontmatter holding title, tags, timestamps, and tombstone (ADR-0016 §5).
//! - Artifact notes are individual files at `artifact/notes/<note_id>.yaml`.

use std::{collections::HashSet, fs, path::Path};

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::store::{
  model::primitives::Id,
  sync::{Error, digest, paths, yaml},
};

const FRONTMATTER_DELIM: &str = "---\n";

/// YAML frontmatter for `.gest/artifact/<id>.md` files.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ArtifactFrontmatter {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  deleted_at: Option<DateTime<Utc>>,
  id: Id,
  title: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  tags: Vec<String>,
  #[serde(default, skip_serializing_if = "JsonValue::is_null")]
  metadata: JsonValue,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  archived_at: Option<DateTime<Utc>>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

/// On-disk wrapper for `.gest/artifact/notes/<note_id>.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ArtifactNoteFile {
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

/// Import every `artifact/*.md` (and `artifact/notes/*.yaml`) file into SQLite.
pub async fn read_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let artifact_dir = gest_dir.join(paths::ARTIFACT_DIR);
  let notes_dir = artifact_dir.join(paths::NOTES_DIR);

  for path in yaml::walk_files(&artifact_dir, "md")? {
    let raw = fs::read_to_string(&path)?;
    let (front, body) = split_frontmatter(&raw)?;
    let front: ArtifactFrontmatter = yaml_serde::from_str(front)?;
    if front.deleted_at.is_some() {
      log::info!("sync import: tombstone for artifact {}", front.id.short());
      conn
        .execute("DELETE FROM artifacts WHERE id = ?1", [front.id.to_string()])
        .await?;
      continue;
    }
    upsert_artifact(conn, project_id, &front, body).await?;
    sync_artifact_tags(conn, &front).await?;
  }

  for path in yaml::walk_files(&notes_dir, "yaml")? {
    let Some(file): Option<ArtifactNoteFile> = yaml::read(&path)? else {
      continue;
    };
    if file.deleted_at.is_some() {
      log::info!("sync import: tombstone for artifact note {}", file.id.short());
      conn
        .execute("DELETE FROM notes WHERE id = ?1", [file.id.to_string()])
        .await?;
      continue;
    }
    upsert_note(conn, &file).await?;
  }

  Ok(())
}

/// Export every artifact and artifact note row to per-entity files.
pub async fn write_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut alive_artifacts: HashSet<String> = HashSet::new();
  let mut alive_notes: HashSet<String> = HashSet::new();

  let mut rows = conn
    .query(
      "SELECT id, archived_at, body, created_at, metadata, title, updated_at \
        FROM artifacts WHERE project_id = ?1 ORDER BY id",
      [project_id.to_string()],
    )
    .await?;
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let archived_at: Option<String> = row.get(1).ok();
    let body: String = row.get(2)?;
    let created_at: String = row.get(3)?;
    let metadata_str: String = row.get(4).unwrap_or_else(|_| "{}".to_string());
    let title: String = row.get(5)?;
    let updated_at: String = row.get(6)?;

    let id: Id = id_str
      .parse()
      .map_err(|e: String| Error::Io(std::io::Error::other(e)))?;
    let metadata: JsonValue = serde_json::from_str(&metadata_str).unwrap_or(JsonValue::Null);
    let tags = load_artifact_tag_labels(conn, &id).await?;

    let front = ArtifactFrontmatter {
      deleted_at: None,
      id: id.clone(),
      title,
      tags,
      metadata,
      archived_at: archived_at.as_deref().map(parse_dt).transpose()?,
      created_at: parse_dt(&created_at)?,
      updated_at: parse_dt(&updated_at)?,
    };
    let serialized = compose_artifact_file(&front, &body)?;
    let path = paths::artifact_path(gest_dir, &id);
    write_artifact_file(conn, project_id, gest_dir, &path, &serialized).await?;
    alive_artifacts.insert(id.to_string());
  }

  // Artifact notes
  let mut rows = conn
    .query(
      "SELECT n.id, n.entity_id, n.author_id, n.body, n.created_at, n.updated_at \
        FROM notes n \
        WHERE n.entity_type = 'artifact' AND n.entity_id IN (SELECT id FROM artifacts WHERE project_id = ?1) \
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
    let file = ArtifactNoteFile {
      deleted_at: None,
      id: id.clone(),
      entity_id,
      author_id,
      body,
      created_at: parse_dt(&created_at)?,
      updated_at: parse_dt(&updated_at)?,
    };
    let path = paths::artifact_note_path(gest_dir, &id);
    yaml::write_cached(conn, project_id, gest_dir, &path, &file).await?;
    alive_notes.insert(id.to_string());
  }

  // Clean up orphaned artifact and artifact-note files. Tombstoned files
  // (frontmatter with `deleted_at` set) are left in place — the tombstone is
  // the delete signal that downstream clones read on import.
  let artifact_dir = gest_dir.join(paths::ARTIFACT_DIR);
  let notes_dir = artifact_dir.join(paths::NOTES_DIR);
  yaml::cleanup_orphans(conn, project_id, gest_dir, &notes_dir, "yaml", &alive_notes).await?;
  for entry in std::fs::read_dir(&artifact_dir).into_iter().flatten().flatten() {
    let path = entry.path();
    if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
      let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
      if !alive_artifacts.contains(stem) && !is_tombstoned_artifact_file(&path) {
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

/// Return `true` if the artifact file at `path` carries a tombstone in its
/// YAML frontmatter. Errors and missing files are treated as "not tombstoned"
/// so cleanup can fall back to its normal path.
fn is_tombstoned_artifact_file(path: &Path) -> bool {
  let Ok(raw) = fs::read_to_string(path) else {
    return false;
  };
  let Ok((front, _body)) = split_frontmatter(&raw) else {
    return false;
  };
  let Ok(value) = yaml_serde::from_str::<yaml_serde::Value>(front) else {
    return false;
  };
  value
    .as_mapping()
    .and_then(|m| m.get(yaml_serde::Value::String("deleted_at".into())))
    .is_some_and(|v| !v.is_null())
}

fn compose_artifact_file(front: &ArtifactFrontmatter, body: &str) -> Result<String, Error> {
  let yaml = yaml_serde::to_string(front)?;
  let mut out = String::new();
  out.push_str(FRONTMATTER_DELIM);
  out.push_str(&yaml);
  if !yaml.ends_with('\n') {
    out.push('\n');
  }
  out.push_str(FRONTMATTER_DELIM);
  out.push_str(body);
  if !body.ends_with('\n') {
    out.push('\n');
  }
  Ok(out)
}

async fn load_artifact_tag_labels(conn: &Connection, artifact_id: &Id) -> Result<Vec<String>, Error> {
  let mut rows = conn
    .query(
      "SELECT t.label FROM entity_tags et JOIN tags t ON t.id = et.tag_id \
        WHERE et.entity_type = 'artifact' AND et.entity_id = ?1 ORDER BY t.label",
      [artifact_id.to_string()],
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

/// Split a markdown file into its YAML frontmatter and the body that follows.
///
/// The returned `front` slice always includes a trailing newline so it can be
/// parsed by `yaml_serde::from_str` directly. The `body` slice is everything
/// after the closing `---` delimiter.
fn split_frontmatter(raw: &str) -> Result<(&str, &str), Error> {
  let trimmed = raw.strip_prefix(FRONTMATTER_DELIM).ok_or_else(|| {
    Error::Io(std::io::Error::other(
      "artifact file is missing the leading `---` frontmatter delimiter",
    ))
  })?;
  let end = trimmed
    .find("\n---")
    .ok_or_else(|| Error::Io(std::io::Error::other("artifact frontmatter is not closed with `---`")))?;
  // Include the newline that terminates the last frontmatter line in `front`.
  let front = &trimmed[..end + 1];
  // Skip past `\n---` to land at the byte after the closing delimiter line.
  let after_delim = &trimmed[end + 4..];
  let body = after_delim.strip_prefix('\n').unwrap_or(after_delim);
  Ok((front, body))
}

async fn sync_artifact_tags(conn: &Connection, front: &ArtifactFrontmatter) -> Result<(), Error> {
  conn
    .execute(
      "DELETE FROM entity_tags WHERE entity_type = 'artifact' AND entity_id = ?1",
      [front.id.to_string()],
    )
    .await?;
  for label in &front.tags {
    let mut rows = conn
      .query("SELECT id FROM tags WHERE label = ?1", [label.clone()])
      .await?;
    let tag_id: String = match rows.next().await? {
      Some(row) => row.get(0)?,
      None => {
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
        "INSERT OR IGNORE INTO entity_tags (entity_type, entity_id, tag_id) VALUES ('artifact', ?1, ?2)",
        [front.id.to_string(), tag_id],
      )
      .await?;
  }
  Ok(())
}

async fn upsert_artifact(
  conn: &Connection,
  project_id: &Id,
  front: &ArtifactFrontmatter,
  body: &str,
) -> Result<(), Error> {
  let metadata_str = serde_json::to_string(&front.metadata).unwrap_or_else(|_| "{}".to_string());
  conn
    .execute(
      "INSERT INTO artifacts (id, project_id, archived_at, body, created_at, metadata, title, updated_at) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
        ON CONFLICT(id) DO UPDATE SET archived_at = ?3, body = ?4, metadata = ?6, title = ?7, updated_at = ?8",
      libsql::params![
        front.id.to_string(),
        project_id.to_string(),
        front.archived_at.map(|d| d.to_rfc3339()),
        body.to_string(),
        front.created_at.to_rfc3339(),
        metadata_str,
        front.title.clone(),
        front.updated_at.to_rfc3339(),
      ],
    )
    .await?;
  Ok(())
}

async fn upsert_note(conn: &Connection, file: &ArtifactNoteFile) -> Result<(), Error> {
  conn
    .execute(
      "INSERT INTO notes (id, entity_id, entity_type, author_id, body, created_at, updated_at) \
        VALUES (?1, ?2, 'artifact', ?3, ?4, ?5, ?6) \
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

/// Write the composed `<frontmatter>\n<body>` artifact file with digest caching.
async fn write_artifact_file(
  conn: &Connection,
  project_id: &Id,
  gest_dir: &Path,
  path: &Path,
  serialized: &str,
) -> Result<(), Error> {
  let new_digest = digest::compute(serialized.as_bytes());
  let relative = paths::relative(gest_dir, path).ok_or_else(|| {
    Error::Io(std::io::Error::other(format!(
      "path {} is outside {}",
      path.display(),
      gest_dir.display()
    )))
  })?;

  if digest::is_unchanged(conn, project_id, &relative, &new_digest).await? {
    return Ok(());
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(path, serialized.as_bytes())?;
  digest::record(conn, project_id, &relative, &new_digest).await?;
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

  async fn insert_artifact(conn: &libsql::Connection, project_id: &Id, title: &str, body: &str) -> Id {
    let id = Id::new();
    conn
      .execute(
        "INSERT INTO artifacts (id, project_id, title, body, created_at, updated_at) \
          VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        libsql::params![
          id.to_string(),
          project_id.to_string(),
          title.to_string(),
          body.to_string(),
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
      let id = insert_artifact(&conn, &pid, "Doomed", "body").await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();

      // Rewrite the file with a deleted_at field at the start of the frontmatter.
      let path = paths::artifact_path(&gest_dir, &id);
      let raw = std::fs::read_to_string(&path).unwrap();
      let modified = raw.replacen("---\n", "---\ndeleted_at: 2026-04-08T12:00:00Z\n", 1);
      std::fs::write(&path, modified).unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT id FROM artifacts WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_roundtrips_artifact_through_disk_preserving_body() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_artifact(&conn, &pid, "Spec", "# Heading\n\nbody text\n").await;
      write_all(&conn, &pid, &gest_dir).await.unwrap();
      conn.execute("DELETE FROM artifacts", ()).await.unwrap();

      read_all(&conn, &pid, &gest_dir).await.unwrap();

      let mut rows = conn
        .query("SELECT title, body FROM artifacts WHERE id = ?1", [id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let title: String = row.get(0).unwrap();
      let body: String = row.get(1).unwrap();
      assert_eq!(title, "Spec");
      assert_eq!(body, "# Heading\n\nbody text\n");
    }
  }

  mod split_frontmatter {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_when_the_leading_delimiter_is_missing() {
      let raw = "title: x\n---\nbody\n";

      let result = split_frontmatter(raw);

      assert!(result.is_err());
    }

    #[test]
    fn it_separates_frontmatter_from_body() {
      let raw = "---\ntitle: x\n---\nhello world\n";

      let (front, body) = split_frontmatter(raw).unwrap();

      assert_eq!(front, "title: x\n");
      assert_eq!(body, "hello world\n");
    }
  }

  mod write_all {
    use super::*;

    #[tokio::test]
    async fn it_does_not_create_a_legacy_index_json_aggregate() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      insert_artifact(&conn, &pid, "Spec", "body").await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      assert!(!gest_dir.join("artifacts/index.json").exists());
      assert!(!gest_dir.join("artifact/index.json").exists());
    }

    #[tokio::test]
    async fn it_writes_artifact_md_with_frontmatter_and_body() {
      let (db, _root, pid, gest_dir) = setup().await;
      let conn = db.connect().await.unwrap();
      let id = insert_artifact(&conn, &pid, "Spec", "# Heading\n\nbody text\n").await;

      write_all(&conn, &pid, &gest_dir).await.unwrap();

      let path = paths::artifact_path(&gest_dir, &id);
      assert!(path.exists());
      let raw = std::fs::read_to_string(&path).unwrap();
      assert!(raw.starts_with("---\n"));
      assert!(raw.contains("title: Spec"));
      assert!(raw.contains("# Heading"));
      assert!(raw.contains("body text"));
    }
  }
}
