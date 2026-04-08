use std::{
  io::Error as IoError,
  path::{Path, PathBuf},
};

use libsql::{Connection, Error as DbError};

use crate::store::model::{Error as ModelError, Project, ProjectWorkspace, primitives::Id};

/// Errors that can occur in project repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A filesystem I/O error occurred.
  #[error(transparent)]
  Io(#[from] IoError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// A YAML serialization error occurred.
  #[error(transparent)]
  Yaml(#[from] yaml_serde::Error),
}

/// Return all projects ordered by creation time (newest first).
pub async fn all(conn: &Connection) -> Result<Vec<Project>, Error> {
  log::debug!("repo::project::all");
  let mut rows = conn
    .query(
      "SELECT id, root, created_at, updated_at FROM projects ORDER BY created_at DESC",
      (),
    )
    .await?;

  let mut projects = Vec::new();
  while let Some(row) = rows.next().await? {
    projects.push(Project::try_from(row)?);
  }
  Ok(projects)
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
      "INSERT INTO project_workspaces (id, path, project_id, created_at, updated_at) \
        VALUES (?1, ?2, ?3, ?4, ?5)",
      [
        ws.id().to_string(),
        ws.path().to_string_lossy().into_owned(),
        ws.project_id().to_string(),
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
      Project::from_synced_parts(stored.id, root.clone(), stored.created_at, stored.updated_at)
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
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("project not found after insert".into())))?;

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
      "SELECT id, root, created_at, updated_at FROM projects WHERE id = ?1",
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
      "SELECT DISTINCT p.id, p.root, p.created_at, p.updated_at \
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
    async fn it_returns_projects_newest_first() {
      let (_store, conn, _tmp) = setup().await;

      let p1 = create(&conn, "/tmp/first").await.unwrap();
      let p2 = create(&conn, "/tmp/second").await.unwrap();

      let projects = all(&conn).await.unwrap();
      assert_eq!(projects.len(), 2);
      assert_eq!(projects[0].id(), p2.id());
      assert_eq!(projects[1].id(), p1.id());
    }

    #[tokio::test]
    async fn it_returns_an_empty_vec_when_empty() {
      let (_store, conn, _tmp) = setup().await;

      let projects = all(&conn).await.unwrap();
      assert_eq!(projects.len(), 0);
    }
  }

  mod create {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_persists_the_project() {
      let (_store, conn, _tmp) = setup().await;

      let created = create(&conn, "/tmp/created").await.unwrap();
      assert_eq!(created.root(), &PathBuf::from("/tmp/created"));
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
    async fn it_does_not_write_yaml_when_no_gest_dir() {
      let tmp = tempfile::tempdir().unwrap();
      let root = tmp.path().to_path_buf();

      let (_store, conn, _tmp) = setup().await;
      create(&conn, &root).await.unwrap();

      assert!(!root.join(".gest/project.yaml").exists());
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
        ws.path().to_string_lossy().into_owned(),
        ws.project_id().to_string(),
        ws.created_at().to_rfc3339(),
        ws.updated_at().to_rfc3339(),
      ];
      conn
        .execute(
          "INSERT INTO project_workspaces (id, path, project_id, created_at, updated_at) \
          VALUES (?1, ?2, ?3, ?4, ?5)",
          params,
        )
        .await
        .unwrap();

      let found = find_by_path(&conn, Path::new("/tmp/my-workspace")).await.unwrap();
      assert_eq!(found.as_ref().map(|p| p.id()), Some(project.id()));
    }

    #[tokio::test]
    async fn it_returns_none_when_no_match() {
      let (_store, conn, _tmp) = setup().await;

      let found = find_by_path(&conn, Path::new("/does/not/exist")).await.unwrap();
      assert_eq!(found, None);
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
}
