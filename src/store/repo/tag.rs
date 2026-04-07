use libsql::{Connection, Error as DbError};

use crate::store::model::{
  Error as ModelError, Tag,
  primitives::{EntityType, Id},
};

/// Errors that can occur in tag repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
}

/// Return all tags ordered by label.
pub async fn all(conn: &Connection) -> Result<Vec<Tag>, Error> {
  let mut rows = conn.query("SELECT id, label FROM tags ORDER BY label", ()).await?;

  let mut tags = Vec::new();
  while let Some(row) = rows.next().await? {
    tags.push(Tag::try_from(row)?);
  }
  Ok(tags)
}

/// Attach a tag to an entity. Creates the tag if it doesn't exist.
pub async fn attach(conn: &Connection, entity_type: EntityType, entity_id: &Id, label: &str) -> Result<Tag, Error> {
  let tag = find_or_create(conn, label).await?;
  conn
    .execute(
      "INSERT OR IGNORE INTO entity_tags (entity_type, entity_id, tag_id) VALUES (?1, ?2, ?3)",
      [entity_type.to_string(), entity_id.to_string(), tag.id().to_string()],
    )
    .await?;
  Ok(tag)
}

/// Return all distinct tags attached to entities of the given type, ordered by label.
pub async fn by_entity_type(conn: &Connection, entity_type: EntityType) -> Result<Vec<Tag>, Error> {
  let mut rows = conn
    .query(
      "SELECT DISTINCT t.id, t.label FROM tags t \
        INNER JOIN entity_tags et ON et.tag_id = t.id \
        WHERE et.entity_type = ?1 \
        ORDER BY t.label",
      [entity_type.to_string()],
    )
    .await?;

  let mut tags = Vec::new();
  while let Some(row) = rows.next().await? {
    tags.push(Tag::try_from(row)?);
  }
  Ok(tags)
}

/// Create a new tag with the given label.
pub async fn create(conn: &Connection, tag: &Tag) -> Result<Tag, Error> {
  conn
    .execute(
      "INSERT INTO tags (id, label) VALUES (?1, ?2)",
      [tag.id().to_string(), tag.label().to_string()],
    )
    .await?;

  find_by_id(conn, tag.id().clone())
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("tag not found after insert".into())))
}

/// Detach a tag from an entity. Does not delete the tag itself.
pub async fn detach(conn: &Connection, entity_type: EntityType, entity_id: &Id, label: &str) -> Result<bool, Error> {
  let Some(tag) = find_by_label(conn, label).await? else {
    return Ok(false);
  };
  let affected = conn
    .execute(
      "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2 AND tag_id = ?3",
      [entity_type.to_string(), entity_id.to_string(), tag.id().to_string()],
    )
    .await?;
  Ok(affected > 0)
}

/// Detach all tags from an entity. Does not delete the tags themselves.
pub async fn detach_all(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<u64, Error> {
  let affected = conn
    .execute(
      "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2",
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;
  Ok(affected)
}

/// Find a tag by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Tag>, Error> {
  let id = id.into();
  let mut rows = conn
    .query("SELECT id, label FROM tags WHERE id = ?1", [id.to_string()])
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Tag::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find a tag by its label.
pub async fn find_by_label(conn: &Connection, label: &str) -> Result<Option<Tag>, Error> {
  let mut rows = conn
    .query("SELECT id, label FROM tags WHERE label = ?1", [label.to_string()])
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Tag::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find an existing tag by label or create a new one.
pub async fn find_or_create(conn: &Connection, label: &str) -> Result<Tag, Error> {
  if let Some(existing) = find_by_label(conn, label).await? {
    return Ok(existing);
  }
  let tag = Tag::new(label);
  create(conn, &tag).await
}

/// Return all tag labels for a specific entity.
pub async fn for_entity(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<Vec<String>, Error> {
  let mut rows = conn
    .query(
      "SELECT t.label FROM tags t \
        INNER JOIN entity_tags et ON et.tag_id = t.id \
        WHERE et.entity_type = ?1 AND et.entity_id = ?2 \
        ORDER BY t.label",
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut labels = Vec::new();
  while let Some(row) = rows.next().await? {
    let label: String = row.get(0)?;
    labels.push(label);
  }
  Ok(labels)
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{self, Db, model::Project};

  async fn setup() -> (Arc<Db>, Connection, TempDir) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    (store, conn, tmp)
  }

  async fn create_project(conn: &Connection) -> Id {
    let project = Project::new("/tmp/test".into());
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
    project.id().clone()
  }

  async fn create_task(conn: &Connection, project_id: &Id) -> Id {
    let task_id = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [task_id.to_string(), project_id.to_string(), "Test task".to_string()],
      )
      .await
      .unwrap();
    task_id
  }

  mod all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_tags_sorted_by_label() {
      let (_store, conn, _tmp) = setup().await;

      let z = Tag::new("zebra");
      let a = Tag::new("alpha");
      create(&conn, &z).await.unwrap();
      create(&conn, &a).await.unwrap();

      let tags = all(&conn).await.unwrap();
      assert_eq!(tags.len(), 2);
      assert_eq!(tags[0].label(), "alpha");
      assert_eq!(tags[1].label(), "zebra");
    }
  }

  mod attach_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_tag_and_attaches() {
      let (_store, conn, _tmp) = setup().await;
      let project_id = create_project(&conn).await;
      let task_id = create_task(&conn, &project_id).await;

      let tag = attach(&conn, EntityType::Task, &task_id, "urgent").await.unwrap();
      assert_eq!(tag.label(), "urgent");

      let labels = for_entity(&conn, EntityType::Task, &task_id).await.unwrap();
      assert_eq!(labels, vec!["urgent"]);
    }

    #[tokio::test]
    async fn it_does_not_duplicate_attachment() {
      let (_store, conn, _tmp) = setup().await;
      let project_id = create_project(&conn).await;
      let task_id = create_task(&conn, &project_id).await;

      attach(&conn, EntityType::Task, &task_id, "urgent").await.unwrap();
      attach(&conn, EntityType::Task, &task_id, "urgent").await.unwrap();

      let labels = for_entity(&conn, EntityType::Task, &task_id).await.unwrap();
      assert_eq!(labels, vec!["urgent"]);
    }
  }

  mod detach_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_removes_tag_from_entity() {
      let (_store, conn, _tmp) = setup().await;
      let project_id = create_project(&conn).await;
      let task_id = create_task(&conn, &project_id).await;

      attach(&conn, EntityType::Task, &task_id, "remove-me").await.unwrap();
      let removed = detach(&conn, EntityType::Task, &task_id, "remove-me").await.unwrap();

      assert!(removed);

      let labels = for_entity(&conn, EntityType::Task, &task_id).await.unwrap();
      assert_eq!(labels.len(), 0);
    }

    #[tokio::test]
    async fn it_returns_false_when_tag_not_found() {
      let (_store, conn, _tmp) = setup().await;
      let project_id = create_project(&conn).await;
      let task_id = create_task(&conn, &project_id).await;

      let removed = detach(&conn, EntityType::Task, &task_id, "nonexistent").await.unwrap();
      assert!(!removed);
    }
  }
}
