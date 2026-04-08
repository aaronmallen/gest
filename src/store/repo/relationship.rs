use chrono::Utc;
use libsql::{Connection, Error as DbError};

use crate::store::model::{
  Error as ModelError,
  primitives::{EntityType, Id, RelationshipType},
  relationship::Model,
};

/// Errors that can occur in relationship repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
}

const SELECT_COLUMNS: &str = "id, rel_type, source_id, source_type, target_id, target_type, created_at, updated_at";

/// Create a new relationship between two entities.
pub async fn create(
  conn: &Connection,
  rel_type: RelationshipType,
  source_type: EntityType,
  source_id: &Id,
  target_type: EntityType,
  target_id: &Id,
) -> Result<Model, Error> {
  log::debug!("repo::relationship::create");
  let id = Id::new();
  let now = Utc::now().to_rfc3339();
  conn
    .execute(
      &format!("INSERT INTO relationships ({SELECT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"),
      [
        id.to_string(),
        rel_type.to_string(),
        source_id.to_string(),
        source_type.to_string(),
        target_id.to_string(),
        target_type.to_string(),
        now.clone(),
        now,
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("relationship not found after insert".into())))
}

/// Delete a relationship by its ID. Returns true if deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  log::debug!("repo::relationship::delete");
  let affected = conn
    .execute("DELETE FROM relationships WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find a relationship by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  log::debug!("repo::relationship::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM relationships WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Return all relationships where the entity is either source or target.
pub async fn for_entity(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<Vec<Model>, Error> {
  log::debug!("repo::relationship::for_entity");
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM relationships \
          WHERE (source_type = ?1 AND source_id = ?2) \
          OR (target_type = ?1 AND target_id = ?2)"
      ),
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut relationships = Vec::new();
  while let Some(row) = rows.next().await? {
    relationships.push(Model::try_from(row)?);
  }
  Ok(relationships)
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db, model::Project};

  async fn setup() -> (Arc<Db>, Connection, TempDir, Id, Id) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/rel-test".into());
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
    let pid = project.id().clone();

    let t1 = Id::new();
    let t2 = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [t1.to_string(), pid.to_string(), "Task 1".to_string()],
      )
      .await
      .unwrap();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [t2.to_string(), pid.to_string(), "Task 2".to_string()],
      )
      .await
      .unwrap();
    (store, conn, tmp, t1, t2)
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_a_relationship() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      let rel = create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      assert_eq!(rel.rel_type(), RelationshipType::Blocks);
      assert_eq!(rel.source_id(), &t1);
      assert_eq!(rel.target_id(), &t2);
    }
  }

  mod delete_fn {
    use super::*;

    #[tokio::test]
    async fn it_deletes_a_relationship() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      let rel = create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let deleted = delete(&conn, rel.id()).await.unwrap();

      assert!(deleted);
    }
  }

  mod for_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_relationships_where_entity_is_source_or_target() {
      let (_store, conn, _tmp, t1, t2) = setup().await;

      create(
        &conn,
        RelationshipType::Blocks,
        EntityType::Task,
        &t1,
        EntityType::Task,
        &t2,
      )
      .await
      .unwrap();

      let rels_t1 = for_entity(&conn, EntityType::Task, &t1).await.unwrap();

      assert_eq!(rels_t1.len(), 1);

      let rels_t2 = for_entity(&conn, EntityType::Task, &t2).await.unwrap();

      assert_eq!(rels_t2.len(), 1);
    }
  }
}
