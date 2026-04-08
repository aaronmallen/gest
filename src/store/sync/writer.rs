use std::{collections::BTreeMap, path::Path};

use chrono::Utc;
use libsql::Connection;

use super::{Error, digest};
use crate::store::model::{artifact, iteration, note, primitives::Id, task};

/// Export all project data to the `.gest/` directory as JSON and markdown files.
pub async fn export_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  log::debug!("sync export: project={} dir={}", project_id.short(), gest_dir.display());
  std::fs::create_dir_all(gest_dir)?;
  std::fs::create_dir_all(gest_dir.join("artifacts"))?;

  export_tasks(conn, project_id, gest_dir).await?;
  export_artifacts(conn, project_id, gest_dir).await?;
  export_iterations(conn, project_id, gest_dir).await?;
  export_notes(conn, project_id, gest_dir).await?;

  log::debug!(
    "sync export: complete project={} dir={}",
    project_id.short(),
    gest_dir.display()
  );
  Ok(())
}

/// Derive a stable filename from an artifact ID: first 8 characters of the encoded ID.
fn artifact_filename(id: &Id) -> String {
  format!("{}.md", id.short())
}

async fn export_artifacts(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut rows = conn
    .query(
      "SELECT id, project_id, archived_at, body, created_at, \
        metadata, title, updated_at \
        FROM artifacts WHERE project_id = ?1 ORDER BY created_at DESC",
      [project_id.to_string()],
    )
    .await?;

  let mut artifacts = Vec::new();
  while let Some(row) = rows.next().await? {
    artifacts.push(artifact::Model::try_from(row)?);
  }

  // Write artifacts metadata JSON (without body, since body is #[serde(skip)])
  let json = serde_json::to_string_pretty(&artifacts)?;
  let path = gest_dir.join("artifacts.json");
  write_if_changed(conn, project_id, &path, json.as_bytes()).await?;

  // Write each artifact body as a standalone markdown file, keyed by ID prefix
  let artifacts_dir = gest_dir.join("artifacts");
  let mut index: BTreeMap<String, String> = BTreeMap::new();
  for artifact in &artifacts {
    let filename = artifact_filename(artifact.id());
    let md_path = artifacts_dir.join(&filename);
    log::debug!(
      "sync export: artifact {} -> {}",
      artifact.id().short(),
      md_path.display()
    );
    write_if_changed(conn, project_id, &md_path, artifact.body().as_bytes()).await?;
    index.insert(filename, artifact.title().to_string());
  }

  // Write an index mapping filenames to titles so humans can find files
  let index_json = serde_json::to_string_pretty(&index)?;
  let index_path = artifacts_dir.join("index.json");
  write_if_changed(conn, project_id, &index_path, index_json.as_bytes()).await?;

  log::debug!("sync export: wrote {} artifacts", artifacts.len());
  Ok(())
}

async fn export_iterations(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut rows = conn
    .query(
      "SELECT id, project_id, completed_at, created_at, description, \
        metadata, status, title, updated_at \
        FROM iterations WHERE project_id = ?1 ORDER BY created_at DESC",
      [project_id.to_string()],
    )
    .await?;

  let mut iterations = Vec::new();
  while let Some(row) = rows.next().await? {
    iterations.push(iteration::Model::try_from(row)?);
  }

  let json = serde_json::to_string_pretty(&iterations)?;
  let path = gest_dir.join("iterations.json");
  write_if_changed(conn, project_id, &path, json.as_bytes()).await?;

  log::debug!("sync export: wrote {} iterations", iterations.len());
  Ok(())
}

async fn export_notes(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  // Export task notes
  let mut rows = conn
    .query(
      "SELECT n.id, n.entity_id, n.entity_type, n.author_id, \
        n.body, n.created_at, n.updated_at \
        FROM notes n \
        WHERE n.entity_type = 'task' AND n.entity_id IN ( \
          SELECT id FROM tasks WHERE project_id = ?1 \
        ) ORDER BY n.created_at DESC",
      [project_id.to_string()],
    )
    .await?;

  let mut task_notes = Vec::new();
  while let Some(row) = rows.next().await? {
    task_notes.push(note::Model::try_from(row)?);
  }

  let json = serde_json::to_string_pretty(&task_notes)?;
  let path = gest_dir.join("task_notes.json");
  write_if_changed(conn, project_id, &path, json.as_bytes()).await?;
  log::debug!("sync export: wrote {} task notes", task_notes.len());

  // Export artifact notes
  let mut rows = conn
    .query(
      "SELECT n.id, n.entity_id, n.entity_type, n.author_id, \
        n.body, n.created_at, n.updated_at \
        FROM notes n \
        WHERE n.entity_type = 'artifact' AND n.entity_id IN ( \
          SELECT id FROM artifacts WHERE project_id = ?1 \
        ) ORDER BY n.created_at DESC",
      [project_id.to_string()],
    )
    .await?;

  let mut artifact_notes = Vec::new();
  while let Some(row) = rows.next().await? {
    artifact_notes.push(note::Model::try_from(row)?);
  }

  let json = serde_json::to_string_pretty(&artifact_notes)?;
  let path = gest_dir.join("artifact_notes.json");
  write_if_changed(conn, project_id, &path, json.as_bytes()).await?;
  log::debug!("sync export: wrote {} artifact notes", artifact_notes.len());

  Ok(())
}

async fn export_tasks(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  let mut rows = conn
    .query(
      "SELECT id, project_id, assigned_to, created_at, description, \
        metadata, priority, resolved_at, status, title, updated_at \
        FROM tasks WHERE project_id = ?1 ORDER BY created_at DESC",
      [project_id.to_string()],
    )
    .await?;

  let mut tasks = Vec::new();
  while let Some(row) = rows.next().await? {
    tasks.push(task::Model::try_from(row)?);
  }

  let json = serde_json::to_string_pretty(&tasks)?;
  let path = gest_dir.join("tasks.json");
  write_if_changed(conn, project_id, &path, json.as_bytes()).await?;

  log::debug!("sync export: wrote {} tasks", tasks.len());
  Ok(())
}

/// Write content to a file only if the digest has changed since last sync.
async fn write_if_changed(conn: &Connection, project_id: &Id, path: &Path, content: &[u8]) -> Result<(), Error> {
  let new_digest = digest::compute(content);
  let path_str = path.to_string_lossy();

  // Check if the digest matches what we last wrote
  let mut rows = conn
    .query(
      "SELECT digest FROM sync_digests WHERE file_path = ?1 AND project_id = ?2",
      [path_str.as_ref().to_string(), project_id.to_string()],
    )
    .await?;

  if let Some(row) = rows.next().await? {
    let old_digest: String = row.get(0)?;
    if old_digest == new_digest {
      log::trace!("sync export: skipped {} (digest unchanged)", path.display());
      return Ok(());
    }
  }

  // Write the file and update the digest
  log::debug!("sync export: writing {}", path.display());
  std::fs::write(path, content)?;

  conn
    .execute(
      "INSERT INTO sync_digests (file_path, project_id, digest, synced_at) \
        VALUES (?1, ?2, ?3, ?4) \
        ON CONFLICT(file_path) DO UPDATE SET digest = ?3, synced_at = ?4",
      [
        path_str.as_ref().to_string(),
        project_id.to_string(),
        new_digest,
        Utc::now().to_rfc3339(),
      ],
    )
    .await?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod artifact_filename {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_uses_id_short_prefix() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      assert_eq!(artifact_filename(&id), "zyxwvuts.md");
    }

    #[test]
    fn it_produces_consistent_output() {
      let id: Id = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".parse().unwrap();
      let a = artifact_filename(&id);
      let b = artifact_filename(&id);
      assert_eq!(a, b);
    }
  }

  mod index_roundtrip {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_through_json() {
      let mut index: BTreeMap<String, String> = BTreeMap::new();
      index.insert("zyxwvuts.md".to_string(), "My Design Doc".to_string());
      index.insert("kkkkkkkk.md".to_string(), "API Spec".to_string());

      let json = serde_json::to_string_pretty(&index).unwrap();
      let parsed: BTreeMap<String, String> = serde_json::from_str(&json).unwrap();

      assert_eq!(index, parsed);
    }

    #[test]
    fn it_maps_filename_to_title() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let filename = artifact_filename(&id);

      let mut index: BTreeMap<String, String> = BTreeMap::new();
      index.insert(filename.clone(), "My Artifact Title".to_string());

      let json = serde_json::to_string_pretty(&index).unwrap();
      let parsed: BTreeMap<String, String> = serde_json::from_str(&json).unwrap();

      assert_eq!(parsed.get(&filename).unwrap(), "My Artifact Title");
    }
  }
}
