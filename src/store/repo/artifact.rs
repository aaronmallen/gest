use chrono::Utc;
use libsql::{Connection, Error as DbError, Value};

use crate::store::model::{
  Error as ModelError,
  artifact::{Filter, Model, New, Patch},
  primitives::Id,
};

/// Errors that can occur in artifact repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// The requested entity was not found.
  #[error("artifact not found: {0}")]
  NotFound(String),
}

const SELECT_COLUMNS: &str = "\
  id, project_id, archived_at, body, created_at, \
  metadata, title, updated_at";

/// Return artifacts for a project, applying the given filter.
pub async fn all(conn: &Connection, project_id: &Id, filter: &Filter) -> Result<Vec<Model>, Error> {
  let mut conditions = vec!["project_id = ?1".to_string()];
  let mut params: Vec<Value> = vec![Value::from(project_id.to_string())];
  let idx = 2;

  if filter.only_archived {
    conditions.push("archived_at IS NOT NULL".to_string());
  } else if !filter.all {
    conditions.push("archived_at IS NULL".to_string());
  }

  if let Some(tag) = &filter.tag {
    conditions.push(format!(
      "id IN (SELECT et.entity_id FROM entity_tags et \
        INNER JOIN tags t ON t.id = et.tag_id \
        WHERE et.entity_type = 'artifact' AND t.label = ?{idx})"
    ));
    params.push(Value::from(tag.clone()));
  }

  let where_clause = conditions.join(" AND ");
  let sql = format!("SELECT {SELECT_COLUMNS} FROM artifacts WHERE {where_clause} ORDER BY created_at DESC");

  let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;
  let mut artifacts = Vec::new();
  while let Some(row) = rows.next().await? {
    artifacts.push(Model::try_from(row)?);
  }
  Ok(artifacts)
}

/// Archive an artifact by setting its archived_at timestamp.
pub async fn archive(conn: &Connection, id: &Id) -> Result<Model, Error> {
  let now = Utc::now();
  let affected = conn
    .execute(
      "UPDATE artifacts SET archived_at = ?1, updated_at = ?2 WHERE id = ?3",
      [now.to_rfc3339(), now.to_rfc3339(), id.to_string()],
    )
    .await?;

  if affected == 0 {
    return Err(Error::NotFound(id.short()));
  }

  find_by_id(conn, id.clone())
    .await?
    .ok_or_else(|| Error::NotFound(id.short()))
}

/// Create a new artifact in the given project.
pub async fn create(conn: &Connection, project_id: &Id, new: &New) -> Result<Model, Error> {
  let id = Id::new();
  let now = Utc::now();
  let metadata = new
    .metadata
    .as_ref()
    .map(|m| m.to_string())
    .unwrap_or_else(|| "{}".to_string());

  conn
    .execute(
      &format!(
        "INSERT INTO artifacts ({SELECT_COLUMNS}) \
          VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)"
      ),
      libsql::params![
        id.to_string(),
        project_id.to_string(),
        new.body.clone(),
        now.to_rfc3339(),
        metadata,
        new.title.clone(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("artifact not found after insert".into())))
}

/// Delete an artifact by its ID. Returns true if the artifact was deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  let affected = conn
    .execute("DELETE FROM artifacts WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find an artifact by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM artifacts WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Update an existing artifact with the given patch.
pub async fn update(conn: &Connection, id: &Id, patch: &Patch) -> Result<Model, Error> {
  let now = Utc::now();
  let mut sets = vec!["updated_at = ?1".to_string()];
  let mut params: Vec<Value> = vec![Value::from(now.to_rfc3339())];
  let mut idx = 2;

  if let Some(title) = &patch.title {
    sets.push(format!("title = ?{idx}"));
    params.push(Value::from(title.clone()));
    idx += 1;
  }

  if let Some(body) = &patch.body {
    sets.push(format!("body = ?{idx}"));
    params.push(Value::from(body.clone()));
    idx += 1;
  }

  if let Some(metadata) = &patch.metadata {
    sets.push(format!("metadata = ?{idx}"));
    params.push(Value::from(metadata.to_string()));
    idx += 1;
  }

  let set_clause = sets.join(", ");
  params.push(Value::from(id.to_string()));
  let sql = format!("UPDATE artifacts SET {set_clause} WHERE id = ?{idx}");

  let affected = conn.execute(&sql, libsql::params_from_iter(params)).await?;

  if affected == 0 {
    return Err(Error::NotFound(id.short()));
  }

  find_by_id(conn, id.clone())
    .await?
    .ok_or_else(|| Error::NotFound(id.short()))
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db, model::Project};

  async fn setup() -> (Arc<Db>, Connection, TempDir, Id) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/artifact-test".into());
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
      .await
      .unwrap();
    let project_id = project.id().clone();
    (store, conn, tmp, project_id)
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_an_artifact() {
      let (_store, conn, _tmp, pid) = setup().await;

      let new = New {
        body: "# Spec\nSome content".into(),
        title: "My spec".into(),
        ..Default::default()
      };
      let artifact = create(&conn, &pid, &new).await.unwrap();

      assert_eq!(artifact.title(), "My spec");
      assert_eq!(artifact.body(), "# Spec\nSome content");
      assert!(!artifact.is_archived());
    }
  }

  mod archive_fn {
    use super::*;

    #[tokio::test]
    async fn it_archives_an_artifact() {
      let (_store, conn, _tmp, pid) = setup().await;
      let artifact = create(
        &conn,
        &pid,
        &New {
          title: "To archive".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let archived = archive(&conn, artifact.id()).await.unwrap();

      assert!(archived.is_archived());
    }
  }

  mod all_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_excludes_archived_by_default() {
      let (_store, conn, _tmp, pid) = setup().await;

      create(
        &conn,
        &pid,
        &New {
          title: "Active".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let to_archive = create(
        &conn,
        &pid,
        &New {
          title: "Archive me".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      archive(&conn, to_archive.id()).await.unwrap();

      let artifacts = all(&conn, &pid, &Filter::default()).await.unwrap();

      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title(), "Active");
    }
  }
}
