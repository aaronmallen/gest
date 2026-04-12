use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use libsql::Connection;

use crate::store::{
  Error,
  model::{
    Project, ProjectWorkspace,
    primitives::{EntityType, Id},
  },
  repo::entity,
  sync::{paths, tombstone},
};

/// Per-table counts of rows removed by a [`delete`] call.
///
/// Returned so that callers (e.g. the CLI) can render cascade counts in a
/// confirmation prompt.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeleteSummary {
  /// Number of owned artifacts removed.
  pub artifacts: usize,
  /// Number of owned iterations removed.
  pub iterations: usize,
  /// Number of attached notes removed (across all entities).
  pub notes: usize,
  /// Number of relationships removed (across all entities).
  pub relationships: usize,
  /// Number of entity_tags attachments removed (across all entities).
  pub tags: usize,
  /// Number of owned tasks removed.
  pub tasks: usize,
}

/// Return projects ordered by creation time (newest first).
///
/// When `include_archived` is false, only active (non-archived) projects are
/// returned. Pass `true` to include archived projects in the result set.
pub async fn all(conn: &Connection, include_archived: bool) -> Result<Vec<Project>, Error> {
  log::debug!("repo::project::all include_archived={include_archived}");
  let sql = if include_archived {
    "SELECT id, root, archived_at, created_at, updated_at FROM projects ORDER BY created_at DESC"
  } else {
    "SELECT id, root, archived_at, created_at, updated_at FROM projects WHERE archived_at IS NULL ORDER BY created_at DESC"
  };
  let mut rows = conn.query(sql, ()).await?;

  let mut projects = Vec::new();
  while let Some(row) = rows.next().await? {
    projects.push(Project::try_from(row)?);
  }
  Ok(projects)
}

/// Soft-archive a project by setting `archived_at` to now and deleting all
/// associated workspace rows in a single operation.
pub async fn archive(conn: &Connection, id: &Id) -> Result<(), Error> {
  log::debug!("repo::project::archive");
  let now = Utc::now().to_rfc3339();
  let id_str = id.to_string();
  conn
    .execute(
      "UPDATE projects SET archived_at = ?1, updated_at = ?1 WHERE id = ?2",
      [now.clone(), id_str.clone()],
    )
    .await?;
  conn
    .execute("DELETE FROM project_workspaces WHERE project_id = ?1", [id_str])
    .await?;
  Ok(())
}

/// Attach a workspace path to a project, creating a new [`ProjectWorkspace`].
pub async fn attach_workspace(
  conn: &Connection,
  project_id: &Id,
  path: impl Into<PathBuf>,
) -> Result<ProjectWorkspace, Error> {
  log::debug!("repo::project::attach_workspace");
  let ws = ProjectWorkspace::new(path.into(), project_id.clone());
  conn
    .execute(
      "INSERT INTO project_workspaces (id, project_id, path, created_at, updated_at) \
        VALUES (?1, ?2, ?3, ?4, ?5)",
      [
        ws.id().to_string(),
        ws.project_id().to_string(),
        ws.path().to_string_lossy().into_owned(),
        ws.created_at().to_rfc3339(),
        ws.updated_at().to_rfc3339(),
      ],
    )
    .await?;
  Ok(ws)
}

/// Create a new project for the given root path.
///
/// If a `.gest/project.yaml` already exists at the root or any ancestor
/// directory, the project's stable id is read from that file (so collaborators
/// share the same id) and a row is inserted with the local checkout's path.
/// Otherwise a fresh project id is generated and `.gest/project.yaml` is
/// written when a `.gest` directory exists.
pub async fn create(conn: &Connection, root: impl Into<PathBuf>) -> Result<Project, Error> {
  log::debug!("repo::project::create");
  let root = root.into();
  let gest_dir = find_gest_dir(&root);

  let project = match gest_dir.as_ref().map(|d| d.join("project.yaml")) {
    Some(path) if path.is_file() => {
      let contents = std::fs::read_to_string(&path)?;
      let stored: ProjectFile = yaml_serde::from_str(&contents)?;
      Project::from_synced_parts(stored.id, root.clone(), None, stored.created_at, stored.updated_at)
    }
    _ => Project::new(root),
  };

  conn
    .execute(
      "INSERT INTO projects (id, root, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
      [
        project.id().to_string(),
        project.root().to_string_lossy().into_owned(),
        project.created_at().to_rfc3339(),
        project.updated_at().to_rfc3339(),
      ],
    )
    .await?;

  let created = find_by_id(conn, project.id().clone())
    .await?
    .ok_or_else(|| Error::InvalidValue("project not found after insert".into()))?;

  if let Some(gest_dir) = gest_dir {
    let stored = ProjectFile {
      id: created.id().clone(),
      created_at: *created.created_at(),
      updated_at: *created.updated_at(),
    };
    let yaml = yaml_serde::to_string(&stored)?;
    std::fs::write(gest_dir.join("project.yaml"), yaml)?;
  }

  Ok(created)
}

/// On-disk shape of `.gest/project.yaml`. Only fields that should travel with
/// the repository are persisted; the local checkout `root` lives in SQLite.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ProjectFile {
  id: Id,
  created_at: chrono::DateTime<chrono::Utc>,
  updated_at: chrono::DateTime<chrono::Utc>,
}

/// Hard-delete a project and every entity it owns, inside a single logical
/// operation.
///
/// This is a non-undoable, cascade delete that:
///
/// 1. Enumerates all tasks, iterations, and artifacts owned by the project.
/// 2. Calls [`entity::delete::delete_with_cascade`] for each so child rows
///    (notes, entity_tags, relationships, iteration_tasks) are cleaned up.
/// 3. Writes tombstone files for every owned entity and the project itself.
/// 4. Deletes `transactions` rows referencing the project.
/// 5. Deletes `project_workspaces` rows referencing the project.
/// 6. Deletes the `projects` row itself.
///
/// No transaction record is opened -- project delete is explicitly
/// non-undoable.
pub async fn delete(
  conn: &Connection,
  id: &Id,
  gest_dir: Option<&Path>,
  deleted_at: DateTime<Utc>,
) -> Result<DeleteSummary, Error> {
  log::debug!("repo::project::delete");

  // 1. Enumerate owned entities.
  let task_ids = collect_entity_ids(conn, "tasks", id).await?;
  let iteration_ids = collect_entity_ids(conn, "iterations", id).await?;
  let artifact_ids = collect_entity_ids(conn, "artifacts", id).await?;

  // 2. Cascade-delete each entity. We use a dummy transaction id since we
  //    are not recording undo events, but delete_with_cascade requires one.
  //    We create a throw-away transaction row that we'll delete in step 4.
  let dummy_tx_id = Id::new();
  conn
    .execute(
      "INSERT INTO transactions (id, project_id, command, created_at) VALUES (?1, ?2, ?3, ?4)",
      [
        dummy_tx_id.to_string(),
        id.to_string(),
        "project delete".to_string(),
        deleted_at.to_rfc3339(),
      ],
    )
    .await?;

  let mut summary = DeleteSummary::default();

  for task_id in &task_ids {
    let report = entity::delete::delete_with_cascade(conn, &dummy_tx_id, EntityType::Task, task_id).await?;
    summary.notes += report.notes;
    summary.tags += report.tags;
    summary.relationships += report.relationships;
    tombstone::tombstone_task(gest_dir, task_id, deleted_at)?;
    invalidate_sync_digest(conn, id, &format!("{}/{}.yaml", paths::TASK_DIR, task_id)).await?;
  }
  summary.tasks = task_ids.len();

  for iteration_id in &iteration_ids {
    let report = entity::delete::delete_with_cascade(conn, &dummy_tx_id, EntityType::Iteration, iteration_id).await?;
    summary.notes += report.notes;
    summary.tags += report.tags;
    summary.relationships += report.relationships;
    tombstone::tombstone_iteration(gest_dir, iteration_id, deleted_at)?;
    invalidate_sync_digest(conn, id, &format!("{}/{}.yaml", paths::ITERATION_DIR, iteration_id)).await?;
  }
  summary.iterations = iteration_ids.len();

  for artifact_id in &artifact_ids {
    let report = entity::delete::delete_with_cascade(conn, &dummy_tx_id, EntityType::Artifact, artifact_id).await?;
    summary.notes += report.notes;
    summary.tags += report.tags;
    summary.relationships += report.relationships;
    tombstone::tombstone_artifact(gest_dir, artifact_id, deleted_at)?;
    invalidate_sync_digest(conn, id, &format!("{}/{}.md", paths::ARTIFACT_DIR, artifact_id)).await?;
  }
  summary.artifacts = artifact_ids.len();

  // 3. Tombstone the project file itself.
  tombstone::tombstone_project(gest_dir, deleted_at)?;

  // 4. Remove all transaction rows (including the dummy) for this project.
  conn
    .execute(
      "DELETE FROM transaction_events WHERE transaction_id IN \
        (SELECT id FROM transactions WHERE project_id = ?1)",
      [id.to_string()],
    )
    .await?;
  conn
    .execute("DELETE FROM transactions WHERE project_id = ?1", [id.to_string()])
    .await?;

  // 5. Delete workspace rows.
  conn
    .execute("DELETE FROM project_workspaces WHERE project_id = ?1", [id.to_string()])
    .await?;

  // 6. Delete the project row itself (sync_digests cascades automatically).
  conn
    .execute("DELETE FROM projects WHERE id = ?1", [id.to_string()])
    .await?;

  Ok(summary)
}

/// Detach the workspace for the given path, removing it from its project.
///
/// Returns `true` if a workspace was deleted, `false` if none matched.
pub async fn detach_workspace(conn: &Connection, path: &Path) -> Result<bool, Error> {
  log::debug!("repo::project::detach_workspace");
  let path_str = path.to_string_lossy();
  let affected = conn
    .execute("DELETE FROM project_workspaces WHERE path = ?1", [path_str.as_ref()])
    .await?;
  Ok(affected > 0)
}

/// Find a project by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Project>, Error> {
  log::debug!("repo::project::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      "SELECT id, root, archived_at, created_at, updated_at FROM projects WHERE id = ?1",
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Project::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find a project by path.
///
/// Matches against both the project's root path and any associated workspace
/// path. Returns the first matching project.
pub async fn find_by_path(conn: &Connection, path: &Path) -> Result<Option<Project>, Error> {
  log::debug!("repo::project::find_by_path");
  let path_str = path.to_string_lossy();

  let mut rows = conn
    .query(
      "SELECT DISTINCT p.id, p.root, p.archived_at, p.created_at, p.updated_at \
      FROM projects p \
      LEFT JOIN project_workspaces pw ON pw.project_id = p.id \
      WHERE p.root = ?1 OR pw.path = ?1 \
      LIMIT 1",
      [path_str.as_ref()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Project::try_from(row)?)),
    None => Ok(None),
  }
}

/// Clear the `archived_at` timestamp on a project, restoring it to active.
pub async fn unarchive(conn: &Connection, id: &Id) -> Result<(), Error> {
  log::debug!("repo::project::unarchive");
  let now = Utc::now().to_rfc3339();
  conn
    .execute(
      "UPDATE projects SET archived_at = NULL, updated_at = ?1 WHERE id = ?2",
      [now, id.to_string()],
    )
    .await?;
  Ok(())
}

/// Collect all entity ids from a table that belong to the given project.
async fn collect_entity_ids(conn: &Connection, table: &str, project_id: &Id) -> Result<Vec<Id>, Error> {
  let sql = format!("SELECT id FROM {table} WHERE project_id = ?1");
  let mut rows = conn.query(&sql, [project_id.to_string()]).await?;
  let mut ids = Vec::new();
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    let id: Id = id_str.parse().map_err(Error::InvalidValue)?;
    ids.push(id);
  }
  Ok(ids)
}

/// Walk from `start` upward through ancestor directories looking for a `.gest`
/// directory. Returns the path to the `.gest` directory if found.
fn find_gest_dir(start: &Path) -> Option<PathBuf> {
  let mut current = start;
  loop {
    let candidate = current.join(".gest");
    if candidate.is_dir() {
      return Some(candidate);
    }
    current = current.parent()?;
  }
}

/// Drop the digest-cache entry for a sync file so follow-up exports don't
/// skip the tombstoned file.
async fn invalidate_sync_digest(conn: &Connection, project_id: &Id, relative: &str) -> Result<(), Error> {
  conn
    .execute(
      "DELETE FROM sync_digests WHERE relative_path = ?1 AND project_id = ?2",
      [relative.to_string(), project_id.to_string()],
    )
    .await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::{path::PathBuf, sync::Arc};

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db, model::ProjectWorkspace};

  async fn setup() -> (Arc<Db>, Connection, TempDir) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    (store, conn, tmp)
  }

  mod all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_excludes_archived_projects_by_default() {
      let (_store, conn, _tmp) = setup().await;

      create(&conn, "/tmp/active").await.unwrap();
      let archived = create(&conn, "/tmp/archived").await.unwrap();
      archive(&conn, archived.id()).await.unwrap();

      let projects = all(&conn, false).await.unwrap();

      assert_eq!(projects.len(), 1);
      assert_eq!(projects[0].root().to_string_lossy(), "/tmp/active");
    }

    #[tokio::test]
    async fn it_includes_archived_projects_when_requested() {
      let (_store, conn, _tmp) = setup().await;

      create(&conn, "/tmp/active").await.unwrap();
      let archived = create(&conn, "/tmp/archived").await.unwrap();
      archive(&conn, archived.id()).await.unwrap();

      let projects = all(&conn, true).await.unwrap();

      assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn it_returns_an_empty_vec_when_empty() {
      let (_store, conn, _tmp) = setup().await;

      let projects = all(&conn, false).await.unwrap();
      assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn it_returns_projects_newest_first() {
      let (_store, conn, _tmp) = setup().await;

      let p1 = create(&conn, "/tmp/first").await.unwrap();
      let p2 = create(&conn, "/tmp/second").await.unwrap();

      let projects = all(&conn, false).await.unwrap();
      assert_eq!(projects.len(), 2);
      assert_eq!(projects[0].id(), p2.id());
      assert_eq!(projects[1].id(), p1.id());
    }
  }

  mod archive {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_deletes_associated_workspaces() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/archive-ws").await.unwrap();
      attach_workspace(&conn, project.id(), "/tmp/ws1").await.unwrap();
      attach_workspace(&conn, project.id(), "/tmp/ws2").await.unwrap();

      archive(&conn, project.id()).await.unwrap();

      let mut rows = conn
        .query(
          "SELECT COUNT(*) FROM project_workspaces WHERE project_id = ?1",
          [project.id().to_string()],
        )
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let count: i64 = row.get(0).unwrap();
      assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn it_sets_archived_at() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/archive-test").await.unwrap();

      assert_eq!(project.archived_at(), &None);

      archive(&conn, project.id()).await.unwrap();

      let found = find_by_id(&conn, project.id().clone()).await.unwrap().unwrap();
      assert!(found.archived_at().is_some());
    }
  }

  mod create {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_does_not_write_yaml_when_no_gest_dir() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();

      let (_store, conn, _tmp) = setup().await;
      create(&conn, &root).await.unwrap();

      assert!(!root.join(".gest/project.yaml").exists());
    }

    #[tokio::test]
    async fn it_persists_the_project() {
      let (_store, conn, _tmp) = setup().await;

      let created = create(&conn, "/tmp/created").await.unwrap();
      assert_eq!(created.root(), &PathBuf::from("/tmp/created"));
    }

    #[tokio::test]
    async fn it_reads_id_from_file_when_project_yaml_exists() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();
      std::fs::create_dir_all(root.join(".gest")).unwrap();

      let existing = Project::new(root.clone());
      let stored = ProjectFile {
        id: existing.id().clone(),
        created_at: *existing.created_at(),
        updated_at: *existing.updated_at(),
      };
      let yaml = yaml_serde::to_string(&stored).unwrap();
      std::fs::write(root.join(".gest/project.yaml"), yaml).unwrap();

      let (_store, conn, _tmp) = setup().await;
      let created = create(&conn, &root).await.unwrap();

      assert_eq!(created.id(), existing.id());
      assert_eq!(created.root(), existing.root());
    }

    #[tokio::test]
    async fn it_rejects_duplicate_root() {
      let (_store, conn, _tmp) = setup().await;

      create(&conn, "/tmp/dup").await.unwrap();
      let err = create(&conn, "/tmp/dup").await.unwrap_err();
      assert!(
        err.to_string().contains("UNIQUE"),
        "expected unique constraint error, got: {err}"
      );
    }

    #[tokio::test]
    async fn it_writes_project_yaml_when_gest_dir_exists() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();
      std::fs::create_dir_all(root.join(".gest")).unwrap();

      let (_store, conn, _tmp) = setup().await;
      let created = create(&conn, &root).await.unwrap();

      let yaml_path = root.join(".gest/project.yaml");
      assert!(yaml_path.exists());

      let contents = std::fs::read_to_string(&yaml_path).unwrap();
      assert!(contents.contains(&format!("id: {}", created.id())));
    }

    #[tokio::test]
    async fn it_writes_project_yaml_when_gest_dir_in_ancestor() {
      let tmp = tempfile::tempdir().unwrap();
      let ancestor = tmp.path().to_path_buf();
      std::fs::create_dir_all(ancestor.join(".gest")).unwrap();
      let child = ancestor.join("sub/dir");
      std::fs::create_dir_all(&child).unwrap();

      let (_store, conn, _tmp) = setup().await;
      create(&conn, &child).await.unwrap();

      let yaml_path = ancestor.join(".gest/project.yaml");
      assert!(yaml_path.exists());
    }
  }

  mod delete_fn {
    use std::fs;

    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::store::{
      model::primitives::{EntityType, RelationshipType},
      repo::{
        artifact as artifact_repo, iteration as iteration_repo, note as note_repo, relationship as relationship_repo,
        tag as tag_repo, task as task_repo, transaction as transaction_repo,
      },
      sync::paths,
    };

    fn sample_deleted_at() -> DateTime<Utc> {
      Utc.with_ymd_and_hms(2026, 4, 8, 12, 0, 0).unwrap()
    }

    async fn row_count(conn: &Connection, sql: &str, params: &[libsql::Value]) -> i64 {
      let mut rows = conn.query(sql, params.to_vec()).await.unwrap();
      let row = rows.next().await.unwrap().unwrap();
      row.get(0).unwrap()
    }

    #[tokio::test]
    async fn it_cascades_all_owned_entities_and_their_children() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/del-cascade").await.unwrap();
      let pid = project.id();

      // Create owned entities.
      let task = task_repo::create(
        &conn,
        pid,
        &crate::store::model::task::New {
          title: "T".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let iteration = iteration_repo::create(
        &conn,
        pid,
        &crate::store::model::iteration::New {
          title: "I".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let artifact = artifact_repo::create(
        &conn,
        pid,
        &crate::store::model::artifact::New {
          title: "A".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      // Add children: notes, tags, relationships, iteration_tasks.
      note_repo::create(
        &conn,
        EntityType::Task,
        task.id(),
        &crate::store::model::note::New {
          body: "note".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();
      tag_repo::attach(&conn, EntityType::Task, task.id(), "urgent")
        .await
        .unwrap();
      iteration_repo::add_task(&conn, iteration.id(), task.id(), 1)
        .await
        .unwrap();
      relationship_repo::create(
        &conn,
        RelationshipType::RelatesTo,
        EntityType::Task,
        task.id(),
        EntityType::Artifact,
        artifact.id(),
      )
      .await
      .unwrap();

      // Create a transaction record so we can verify it gets cleaned up.
      transaction_repo::begin(&conn, pid, "task create").await.unwrap();

      let summary = delete(&conn, pid, None, sample_deleted_at()).await.unwrap();

      assert_eq!(summary.tasks, 1);
      assert_eq!(summary.iterations, 1);
      assert_eq!(summary.artifacts, 1);
      assert_eq!(summary.notes, 1);
      assert_eq!(summary.tags, 1);
      assert_eq!(summary.relationships, 1);

      // All rows gone.
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM projects WHERE id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM tasks WHERE project_id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM iterations WHERE project_id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM artifacts WHERE project_id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM transactions WHERE project_id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM notes WHERE entity_id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM entity_tags WHERE entity_id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM relationships WHERE source_id = ?1 OR target_id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        0
      );
    }

    #[tokio::test]
    async fn it_writes_tombstone_files_for_all_entities() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/del-tomb").await.unwrap();
      let pid = project.id();

      let task = task_repo::create(
        &conn,
        pid,
        &crate::store::model::task::New {
          title: "T".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let iteration = iteration_repo::create(
        &conn,
        pid,
        &crate::store::model::iteration::New {
          title: "I".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let artifact = artifact_repo::create(
        &conn,
        pid,
        &crate::store::model::artifact::New {
          title: "A".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      // Create on-disk files in a temp gest_dir.
      let gest_tmp = tempfile::tempdir().unwrap();
      let gest_dir = gest_tmp.path();

      let task_path = paths::task_path(gest_dir, task.id());
      fs::create_dir_all(task_path.parent().unwrap()).unwrap();
      fs::write(&task_path, "id: placeholder\ntitle: T\n").unwrap();

      let iteration_path = paths::iteration_path(gest_dir, iteration.id());
      fs::create_dir_all(iteration_path.parent().unwrap()).unwrap();
      fs::write(&iteration_path, "id: placeholder\ntitle: I\n").unwrap();

      let artifact_path = paths::artifact_path(gest_dir, artifact.id());
      fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
      fs::write(&artifact_path, "---\nid: placeholder\ntitle: A\n---\nBody.\n").unwrap();

      let project_path = paths::project_path(gest_dir);
      fs::write(
        &project_path,
        &format!(
          "id: {}\ncreated_at: 2026-04-01T00:00:00Z\nupdated_at: 2026-04-01T00:00:00Z\n",
          pid
        ),
      )
      .unwrap();

      delete(&conn, pid, Some(gest_dir), sample_deleted_at()).await.unwrap();

      // All files should contain deleted_at.
      let task_raw = fs::read_to_string(&task_path).unwrap();
      assert!(task_raw.contains("deleted_at:"), "task file should be tombstoned");

      let iteration_raw = fs::read_to_string(&iteration_path).unwrap();
      assert!(
        iteration_raw.contains("deleted_at:"),
        "iteration file should be tombstoned"
      );

      let artifact_raw = fs::read_to_string(&artifact_path).unwrap();
      assert!(
        artifact_raw.contains("deleted_at:"),
        "artifact file should be tombstoned"
      );

      let project_raw = fs::read_to_string(&project_path).unwrap();
      assert!(project_raw.contains("deleted_at:"), "project file should be tombstoned");
    }

    #[tokio::test]
    async fn it_prevents_sync_resurrection_after_delete() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/del-no-resurrect").await.unwrap();
      let pid = project.id().clone();

      let task = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "Ephemeral".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      // Set up a gest_dir with an on-disk task file.
      let gest_tmp = tempfile::tempdir().unwrap();
      let gest_dir = gest_tmp.path();

      let task_path = paths::task_path(gest_dir, task.id());
      fs::create_dir_all(task_path.parent().unwrap()).unwrap();
      fs::write(
        &task_path,
        &format!(
          "id: {}\ntitle: Ephemeral\nstatus: open\ncreated_at: 2026-04-01T00:00:00Z\nupdated_at: 2026-04-01T00:00:00Z\n",
          task.id()
        ),
      )
      .unwrap();

      let project_path = paths::project_path(gest_dir);
      fs::write(
        &project_path,
        &format!(
          "id: {}\ncreated_at: 2026-04-01T00:00:00Z\nupdated_at: 2026-04-01T00:00:00Z\n",
          pid
        ),
      )
      .unwrap();

      delete(&conn, &pid, Some(gest_dir), sample_deleted_at()).await.unwrap();

      // Now simulate a sync import -- the tombstoned files should prevent
      // resurrection.
      crate::store::sync::import(&conn, &pid, gest_dir).await.unwrap();

      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM tasks WHERE id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        0,
        "task must not be resurrected after tombstone"
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM projects WHERE id = ?1",
          &[pid.to_string().into()]
        )
        .await,
        0,
        "project must not be resurrected after tombstone"
      );
    }
  }

  mod find_by_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_none_when_project_does_not_exist() {
      let (_store, conn, _tmp) = setup().await;

      let found = find_by_id(&conn, Id::new()).await.unwrap();
      assert_eq!(found, None);
    }

    #[tokio::test]
    async fn it_returns_the_project_when_project_exists() {
      let (_store, conn, _tmp) = setup().await;
      let created = create(&conn, "/tmp/my-project").await.unwrap();

      let found = find_by_id(&conn, created.id().clone()).await.unwrap();
      assert_eq!(found.as_ref().map(|p| p.id()), Some(created.id()));
    }
  }

  mod find_by_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_none_when_no_match() {
      let (_store, conn, _tmp) = setup().await;

      let found = find_by_path(&conn, Path::new("/does/not/exist")).await.unwrap();
      assert_eq!(found, None);
    }

    #[tokio::test]
    async fn it_returns_the_project_when_matching_root() {
      let (_store, conn, _tmp) = setup().await;
      let created = create(&conn, "/tmp/my-project").await.unwrap();

      let found = find_by_path(&conn, Path::new("/tmp/my-project")).await.unwrap();
      assert_eq!(found.as_ref().map(|p| p.id()), Some(created.id()));
    }

    #[tokio::test]
    async fn it_returns_the_project_when_matching_workspace_path() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/my-project").await.unwrap();

      let ws = ProjectWorkspace::new(PathBuf::from("/tmp/my-workspace"), project.id().clone());
      let params: [String; 5] = [
        ws.id().to_string(),
        ws.project_id().to_string(),
        ws.path().to_string_lossy().into_owned(),
        ws.created_at().to_rfc3339(),
        ws.updated_at().to_rfc3339(),
      ];
      conn
        .execute(
          "INSERT INTO project_workspaces (id, project_id, path, created_at, updated_at) \
          VALUES (?1, ?2, ?3, ?4, ?5)",
          params,
        )
        .await
        .unwrap();

      let found = find_by_path(&conn, Path::new("/tmp/my-workspace")).await.unwrap();
      assert_eq!(found.as_ref().map(|p| p.id()), Some(project.id()));
    }
  }

  mod find_gest_dir_fn {
    use super::*;

    #[test]
    fn it_returns_it_when_gest_dir_at_start() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();
      let gest = root.join(".gest");
      std::fs::create_dir_all(&gest).unwrap();

      assert_eq!(find_gest_dir(&root), Some(gest));
    }

    #[test]
    fn it_returns_it_when_gest_dir_in_ancestor() {
      let tmp = tempfile::tempdir().unwrap();
      let ancestor = tmp.path().to_path_buf();
      let gest = ancestor.join(".gest");
      std::fs::create_dir_all(&gest).unwrap();
      let child = ancestor.join("sub/dir");
      std::fs::create_dir_all(&child).unwrap();

      assert_eq!(find_gest_dir(&child), Some(gest));
    }

    #[test]
    fn it_returns_none_when_no_gest_dir() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();

      assert_eq!(find_gest_dir(&root), None);
    }
  }

  mod unarchive {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_clears_archived_at() {
      let (_store, conn, _tmp) = setup().await;
      let project = create(&conn, "/tmp/unarchive-test").await.unwrap();
      archive(&conn, project.id()).await.unwrap();

      unarchive(&conn, project.id()).await.unwrap();

      let found = find_by_id(&conn, project.id().clone()).await.unwrap().unwrap();
      assert_eq!(found.archived_at(), &None);
    }
  }
}
