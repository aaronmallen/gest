use chrono::Utc;
use libsql::{Connection, Error as DbError, Value};

use crate::store::model::{
  Error as ModelError,
  primitives::Id,
  task::{Filter, Model, New, Patch},
};

/// Errors that can occur in task repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// The requested entity was not found.
  #[error("task not found: {0}")]
  NotFound(String),
}

const SELECT_COLUMNS: &str = "\
  id, project_id, assigned_to, created_at, description, \
  metadata, priority, resolved_at, status, title, updated_at";

/// Return tasks for a project, applying the given filter.
pub async fn all(conn: &Connection, project_id: &Id, filter: &Filter) -> Result<Vec<Model>, Error> {
  let mut conditions = vec!["project_id = ?1".to_string()];
  let mut params: Vec<Value> = vec![Value::from(project_id.to_string())];
  let mut idx = 2;

  if !filter.all {
    conditions.push("status NOT IN ('done', 'cancelled')".to_string());
  }

  if let Some(status) = &filter.status {
    conditions.push(format!("status = ?{idx}"));
    params.push(Value::from(status.to_string()));
    idx += 1;
  }

  if let Some(assigned) = &filter.assigned_to {
    conditions.push(format!("assigned_to IN (SELECT id FROM authors WHERE name = ?{idx})"));
    params.push(Value::from(assigned.clone()));
    idx += 1;
  }

  if let Some(tag) = &filter.tag {
    conditions.push(format!(
      "id IN (SELECT et.entity_id FROM entity_tags et \
        INNER JOIN tags t ON t.id = et.tag_id \
        WHERE et.entity_type = 'task' AND t.label = ?{idx})"
    ));
    params.push(Value::from(tag.clone()));
    let _ = idx;
  }

  let where_clause = conditions.join(" AND ");
  let sql = format!("SELECT {SELECT_COLUMNS} FROM tasks WHERE {where_clause} ORDER BY created_at DESC");

  let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;
  let mut tasks = Vec::new();
  while let Some(row) = rows.next().await? {
    tasks.push(Model::try_from(row)?);
  }
  Ok(tasks)
}

/// Create a new task in the given project.
pub async fn create(conn: &Connection, project_id: &Id, new: &New) -> Result<Model, Error> {
  let id = Id::new();
  let now = Utc::now();
  let status = new.status.unwrap_or_default();
  let resolved_at: Value = if status.is_terminal() {
    Value::from(now.to_rfc3339())
  } else {
    Value::Null
  };
  let metadata = new
    .metadata
    .as_ref()
    .map(|m| m.to_string())
    .unwrap_or_else(|| "{}".to_string());
  let assigned_to: Value = match &new.assigned_to {
    Some(id) => Value::from(id.to_string()),
    None => Value::Null,
  };
  let priority: Value = match new.priority {
    Some(p) => Value::from(p as i64),
    None => Value::Null,
  };

  conn
    .execute(
      &format!(
        "INSERT INTO tasks ({SELECT_COLUMNS}) \
          VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
      ),
      libsql::params![
        id.to_string(),
        project_id.to_string(),
        assigned_to,
        now.to_rfc3339(),
        new.description.clone(),
        metadata,
        priority,
        resolved_at,
        status.to_string(),
        new.title.clone(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("task not found after insert".into())))
}

/// Delete a task by its ID. Returns true if the task was deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  let affected = conn
    .execute("DELETE FROM tasks WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find a task by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM tasks WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Update an existing task with the given patch.
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

  if let Some(description) = &patch.description {
    sets.push(format!("description = ?{idx}"));
    params.push(Value::from(description.clone()));
    idx += 1;
  }

  if let Some(status) = &patch.status {
    sets.push(format!("status = ?{idx}"));
    params.push(Value::from(status.to_string()));
    idx += 1;

    if status.is_terminal() {
      sets.push(format!("resolved_at = ?{idx}"));
      params.push(Value::from(now.to_rfc3339()));
      idx += 1;
    } else {
      sets.push("resolved_at = NULL".to_string());
    }
  }

  if let Some(priority) = &patch.priority {
    sets.push(format!("priority = ?{idx}"));
    params.push(match priority {
      Some(p) => Value::from(*p as i64),
      None => Value::Null,
    });
    idx += 1;
  }

  if let Some(assigned_to) = &patch.assigned_to {
    sets.push(format!("assigned_to = ?{idx}"));
    params.push(match assigned_to {
      Some(a) => Value::from(a.to_string()),
      None => Value::Null,
    });
    idx += 1;
  }

  if let Some(metadata) = &patch.metadata {
    sets.push(format!("metadata = ?{idx}"));
    params.push(Value::from(metadata.to_string()));
    idx += 1;
  }

  let set_clause = sets.join(", ");
  params.push(Value::from(id.to_string()));
  let sql = format!("UPDATE tasks SET {set_clause} WHERE id = ?{idx}");

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
  use crate::store::{
    self, Db,
    model::{Project, primitives::TaskStatus},
  };

  async fn setup() -> (Arc<Db>, Connection, TempDir, Id) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/task-test".into());
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

  mod all {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_non_terminal_tasks_by_default() {
      let (_store, conn, _tmp, pid) = setup().await;

      let open = New {
        title: "Open task".into(),
        ..Default::default()
      };
      let done = New {
        title: "Done task".into(),
        status: Some(TaskStatus::Done),
        ..Default::default()
      };
      create(&conn, &pid, &open).await.unwrap();
      create(&conn, &pid, &done).await.unwrap();

      let tasks = all(&conn, &pid, &Filter::default()).await.unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title(), "Open task");
    }

    #[tokio::test]
    async fn it_returns_all_tasks_when_all_flag_set() {
      let (_store, conn, _tmp, pid) = setup().await;

      let open = New {
        title: "Open".into(),
        ..Default::default()
      };
      let done = New {
        title: "Done".into(),
        status: Some(TaskStatus::Done),
        ..Default::default()
      };
      create(&conn, &pid, &open).await.unwrap();
      create(&conn, &pid, &done).await.unwrap();

      let tasks = all(
        &conn,
        &pid,
        &Filter {
          all: true,
          ..Default::default()
        },
      )
      .await
      .unwrap();
      assert_eq!(tasks.len(), 2);
    }
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_a_task() {
      let (_store, conn, _tmp, pid) = setup().await;

      let new = New {
        title: "My task".into(),
        description: "Do something".into(),
        priority: Some(1),
        ..Default::default()
      };
      let task = create(&conn, &pid, &new).await.unwrap();

      assert_eq!(task.title(), "My task");
      assert_eq!(task.description(), "Do something");
      assert_eq!(task.priority(), Some(1));
      assert_eq!(task.status(), TaskStatus::Open);
      assert!(task.resolved_at().is_none());
    }

    #[tokio::test]
    async fn it_sets_resolved_at_for_terminal_status() {
      let (_store, conn, _tmp, pid) = setup().await;

      let new = New {
        title: "Done".into(),
        status: Some(TaskStatus::Done),
        ..Default::default()
      };
      let task = create(&conn, &pid, &new).await.unwrap();

      assert!(task.resolved_at().is_some());
    }
  }

  mod find_by_id_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_none_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let found = find_by_id(&conn, Id::new()).await.unwrap();
      assert_eq!(found, None);
    }
  }

  mod update_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_updates_title() {
      let (_store, conn, _tmp, pid) = setup().await;
      let task = create(
        &conn,
        &pid,
        &New {
          title: "Old".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let updated = update(
        &conn,
        task.id(),
        &Patch {
          title: Some("New".into()),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      assert_eq!(updated.title(), "New");
    }

    #[tokio::test]
    async fn it_sets_resolved_at_when_completing() {
      let (_store, conn, _tmp, pid) = setup().await;
      let task = create(
        &conn,
        &pid,
        &New {
          title: "Task".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let updated = update(
        &conn,
        task.id(),
        &Patch {
          status: Some(TaskStatus::Done),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      assert!(updated.resolved_at().is_some());
      assert_eq!(updated.status(), TaskStatus::Done);
    }
  }

  mod delete_fn {
    use super::*;

    #[tokio::test]
    async fn it_deletes_a_task() {
      let (_store, conn, _tmp, pid) = setup().await;
      let task = create(
        &conn,
        &pid,
        &New {
          title: "Delete me".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let deleted = delete(&conn, task.id()).await.unwrap();
      assert!(deleted);

      let found = find_by_id(&conn, task.id().clone()).await.unwrap();
      assert!(found.is_none());
    }

    #[tokio::test]
    async fn it_returns_false_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let deleted = delete(&conn, &Id::new()).await.unwrap();
      assert!(!deleted);
    }
  }
}
