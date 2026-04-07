use chrono::Utc;
use libsql::{Connection, Error as DbError, Value};
use serde_json::Value as JsonValue;

use crate::store::model::{
  Error as ModelError,
  event::Model,
  primitives::{EntityType, EventKind, Id},
};

/// Errors that can occur in event repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
}

const SELECT_COLUMNS: &str = "id, entity_id, entity_type, author_id, created_at, data, description, event_type";

/// Record a new event on an entity.
pub async fn create(
  conn: &Connection,
  entity_type: EntityType,
  entity_id: &Id,
  event_type: EventKind,
  data: &JsonValue,
  author_id: Option<&Id>,
  description: Option<&str>,
) -> Result<Model, Error> {
  let id = Id::new();
  let now = Utc::now();
  let author: Value = match author_id {
    Some(a) => Value::from(a.to_string()),
    None => Value::Null,
  };
  let desc: Value = match description {
    Some(d) => Value::from(d.to_string()),
    None => Value::Null,
  };

  conn
    .execute(
      &format!("INSERT INTO events ({SELECT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"),
      libsql::params![
        id.to_string(),
        entity_id.to_string(),
        entity_type.to_string(),
        author,
        now.to_rfc3339(),
        data.to_string(),
        desc,
        event_type.to_string(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("event not found after insert".into())))
}

/// Find an event by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM events WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Return all events for a specific entity, newest first.
pub async fn for_entity(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<Vec<Model>, Error> {
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM events \
          WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at DESC"
      ),
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut events = Vec::new();
  while let Some(row) = rows.next().await? {
    events.push(Model::try_from(row)?);
  }
  Ok(events)
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
    let project = Project::new("/tmp/event-test".into());
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
    let task_id = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [task_id.to_string(), project.id().to_string(), "Test".to_string()],
      )
      .await
      .unwrap();
    (store, conn, tmp, task_id)
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_an_event() {
      let (_store, conn, _tmp, task_id) = setup().await;

      let data = serde_json::json!({"from": "open", "to": "done"});
      let event = create(&conn, EntityType::Task, &task_id, EventKind::Status, &data, None, None)
        .await
        .unwrap();

      assert_eq!(event.event_type(), EventKind::Status);
      assert_eq!(event.entity_type(), EntityType::Task);
    }
  }

  mod for_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_events_for_entity() {
      let (_store, conn, _tmp, task_id) = setup().await;

      let data = serde_json::json!({});
      create(&conn, EntityType::Task, &task_id, EventKind::Status, &data, None, None)
        .await
        .unwrap();
      create(
        &conn,
        EntityType::Task,
        &task_id,
        EventKind::Priority,
        &data,
        None,
        None,
      )
      .await
      .unwrap();

      let events = for_entity(&conn, EntityType::Task, &task_id).await.unwrap();

      assert_eq!(events.len(), 2);
    }
  }
}
