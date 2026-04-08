use chrono::Utc;
use libsql::{Connection, Error as DbError, Value};
use serde_json::Value as JsonValue;

use crate::store::model::{
  Error as ModelError,
  primitives::Id,
  transaction::{Event, Model},
};

/// Errors that can occur in transaction repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// No undoable transaction found.
  #[error("nothing to undo")]
  NothingToUndo,
}

const SELECT_COLUMNS: &str = "id, project_id, author_id, command, undone_at, created_at";

/// Begin a new transaction for the given command.
pub async fn begin(conn: &Connection, project_id: &Id, command: &str) -> Result<Model, Error> {
  log::debug!("repo::transaction::begin");
  begin_with_author(conn, project_id, command, None).await
}

/// Begin a new transaction attributed to a specific author.
pub async fn begin_with_author(
  conn: &Connection,
  project_id: &Id,
  command: &str,
  author_id: Option<&Id>,
) -> Result<Model, Error> {
  log::debug!("repo::transaction::begin_with_author");
  let id = Id::new();
  let now = Utc::now();
  let author: Value = match author_id {
    Some(a) => Value::from(a.to_string()),
    None => Value::Null,
  };
  conn
    .execute(
      "INSERT INTO transactions (id, project_id, author_id, command, created_at) \
        VALUES (?1, ?2, ?3, ?4, ?5)",
      libsql::params![
        id.to_string(),
        project_id.to_string(),
        author,
        command.to_string(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("transaction not found after insert".into())))
}

/// Find a transaction by its ID.
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  log::debug!("repo::transaction::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM transactions WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Return the most recent non-undone transaction for a project.
pub async fn latest_undoable(conn: &Connection, project_id: &Id) -> Result<Option<Model>, Error> {
  log::debug!("repo::transaction::latest_undoable");
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM transactions \
          WHERE project_id = ?1 AND undone_at IS NULL \
          ORDER BY created_at DESC LIMIT 1"
      ),
      [project_id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Return the N most recent non-undone transactions for a project.
pub async fn latest_undoable_n(conn: &Connection, project_id: &Id, n: u32) -> Result<Vec<Model>, Error> {
  log::debug!("repo::transaction::latest_undoable_n");
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM transactions \
          WHERE project_id = ?1 AND undone_at IS NULL \
          ORDER BY created_at DESC LIMIT ?2"
      ),
      libsql::params![project_id.to_string(), n],
    )
    .await?;

  let mut results = Vec::new();
  while let Some(row) = rows.next().await? {
    results.push(Model::try_from(row)?);
  }
  Ok(results)
}

/// Record a change event within a transaction.
///
/// This records the row-level audit metadata needed for undo replay and leaves
/// the timeline-facing semantic fields `NULL`. Use [`record_semantic_event`]
/// instead when the mutation maps to a user-facing activity entry
/// (create/status-change/phase-change/priority-change/archive/complete/cancel).
pub async fn record_event(
  conn: &Connection,
  transaction_id: &Id,
  table_name: &str,
  row_id: &str,
  event_type: &str,
  before_data: Option<&JsonValue>,
) -> Result<(), Error> {
  log::debug!("repo::transaction::record_event");
  record_semantic_event(
    conn,
    transaction_id,
    table_name,
    row_id,
    event_type,
    before_data,
    None,
    None,
    None,
  )
  .await
}

/// Record a change event with semantic timeline metadata.
///
/// `semantic_type`, `old_value`, and `new_value` feed the unified activity
/// timeline. They should be populated for user-facing mutations and left
/// `None` for internal or free-form edits that have no human-readable
/// timeline entry (in which case the plain [`record_event`] is sufficient).
#[allow(clippy::too_many_arguments)]
pub async fn record_semantic_event(
  conn: &Connection,
  transaction_id: &Id,
  table_name: &str,
  row_id: &str,
  event_type: &str,
  before_data: Option<&JsonValue>,
  semantic_type: Option<&str>,
  old_value: Option<&str>,
  new_value: Option<&str>,
) -> Result<(), Error> {
  log::debug!("repo::transaction::record_semantic_event");
  let id = Id::new();
  let before: Value = match before_data {
    Some(d) => Value::from(d.to_string()),
    None => Value::Null,
  };
  let semantic: Value = match semantic_type {
    Some(s) => Value::from(s.to_string()),
    None => Value::Null,
  };
  let old: Value = match old_value {
    Some(v) => Value::from(v.to_string()),
    None => Value::Null,
  };
  let new: Value = match new_value {
    Some(v) => Value::from(v.to_string()),
    None => Value::Null,
  };
  conn
    .execute(
      "INSERT INTO transaction_events \
        (id, transaction_id, before_data, event_type, row_id, table_name, semantic_type, old_value, new_value, created_at) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
      libsql::params![
        id.to_string(),
        transaction_id.to_string(),
        before,
        event_type.to_string(),
        row_id.to_string(),
        table_name.to_string(),
        semantic,
        old,
        new,
        Utc::now().to_rfc3339(),
      ],
    )
    .await?;
  Ok(())
}

/// A semantic event entry for the activity timeline, joined with its parent
/// transaction's author and creation timestamp.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticEvent {
  pub author_id: Option<Id>,
  pub created_at: chrono::DateTime<Utc>,
  pub id: Id,
  pub new_value: Option<String>,
  pub old_value: Option<String>,
  pub row_id: String,
  pub semantic_type: String,
  pub table_name: String,
}

/// Return all semantic events for a given row, ordered by creation time ascending.
///
/// Filters out events with `semantic_type IS NULL`, so free-form edits
/// never appear in the activity timeline.
pub async fn semantic_events_for_row(
  conn: &Connection,
  table_name: &str,
  row_id: &Id,
) -> Result<Vec<SemanticEvent>, Error> {
  log::debug!("repo::transaction::semantic_events_for_row");
  let mut rows = conn
    .query(
      "SELECT te.id, te.row_id, te.table_name, te.semantic_type, te.old_value, te.new_value, \
        te.created_at, t.author_id \
        FROM transaction_events te \
        JOIN transactions t ON t.id = te.transaction_id \
        WHERE te.table_name = ?1 AND te.row_id = ?2 \
          AND te.semantic_type IS NOT NULL \
          AND t.undone_at IS NULL \
        ORDER BY te.created_at ASC",
      libsql::params![table_name.to_string(), row_id.to_string()],
    )
    .await?;

  let mut results = Vec::new();
  while let Some(row) = rows.next().await? {
    let id: String = row.get(0)?;
    let row_id: String = row.get(1)?;
    let table_name: String = row.get(2)?;
    let semantic_type: String = row.get(3)?;
    let old_value: Option<String> = row.get(4)?;
    let new_value: Option<String> = row.get(5)?;
    let created_at: String = row.get(6)?;
    let author_id: Option<String> = row.get(7)?;

    let id: Id = id.parse().map_err(ModelError::InvalidValue)?;
    let author_id = author_id
      .map(|s| s.parse::<Id>())
      .transpose()
      .map_err(ModelError::InvalidValue)?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| ModelError::InvalidValue(e.to_string()))?;

    results.push(SemanticEvent {
      author_id,
      created_at,
      id,
      new_value,
      old_value,
      row_id,
      semantic_type,
      table_name,
    });
  }
  Ok(results)
}

/// Undo a transaction by replaying its events in reverse.
pub async fn undo(conn: &Connection, transaction_id: &Id) -> Result<String, Error> {
  log::debug!("repo::transaction::undo");
  // Get the transaction
  let tx = find_by_id(conn, transaction_id.clone())
    .await?
    .ok_or(Error::NothingToUndo)?;

  // Get all events in reverse order
  let mut rows = conn
    .query(
      "SELECT id, transaction_id, before_data, event_type, row_id, table_name, \
        semantic_type, old_value, new_value, created_at \
        FROM transaction_events WHERE transaction_id = ?1 ORDER BY created_at DESC",
      [transaction_id.to_string()],
    )
    .await?;

  let mut events = Vec::new();
  while let Some(row) = rows.next().await? {
    events.push(Event::try_from(row)?);
  }

  // Replay each event
  for event in &events {
    match event.event_type().as_str() {
      "created" => {
        // Undo a create by deleting the row
        let sql = format!("DELETE FROM {} WHERE id = ?1", event.table_name());
        conn.execute(&sql, [event.row_id().to_string()]).await?;
      }
      "modified" => {
        // Undo a modify by restoring before_data
        if let Some(before) = event.before_data()
          && let Some(obj) = before.as_object()
        {
          let mut sets = Vec::new();
          let mut params: Vec<String> = Vec::new();
          let mut idx = 1;
          for (key, val) in obj {
            if key == "id" {
              continue;
            }
            sets.push(format!("{key} = ?{idx}"));
            params.push(match val {
              JsonValue::String(s) => s.clone(),
              JsonValue::Null => continue,
              other => other.to_string(),
            });
            idx += 1;
          }
          if !sets.is_empty() {
            params.push(event.row_id().to_string());
            let sql = format!(
              "UPDATE {} SET {} WHERE id = ?{idx}",
              event.table_name(),
              sets.join(", ")
            );
            conn
              .execute(&sql, libsql::params_from_iter(params.iter().map(|s| s.as_str())))
              .await?;
          }
        }
      }
      "deleted" => {
        // Undo a delete by re-inserting from before_data
        if let Some(before) = event.before_data()
          && let Some(obj) = before.as_object()
        {
          let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
          let placeholders: Vec<String> = (1..=keys.len()).map(|i| format!("?{i}")).collect();
          let values: Vec<String> = obj
            .values()
            .map(|v| match v {
              JsonValue::String(s) => s.clone(),
              JsonValue::Null => String::new(),
              other => other.to_string(),
            })
            .collect();
          let sql = format!(
            "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
            event.table_name(),
            keys.join(", "),
            placeholders.join(", ")
          );
          conn
            .execute(&sql, libsql::params_from_iter(values.iter().map(|s| s.as_str())))
            .await?;
        }
      }
      _ => {}
    }
  }

  // Mark the transaction as undone
  conn
    .execute(
      "UPDATE transactions SET undone_at = ?1 WHERE id = ?2",
      [Utc::now().to_rfc3339(), transaction_id.to_string()],
    )
    .await?;

  Ok(tx.command().to_string())
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
    let project = Project::new("/tmp/tx-test".into());
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
    (store, conn, tmp, pid)
  }

  mod begin_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_a_transaction() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task create").await.unwrap();

      assert_eq!(tx.command(), "task create");
      assert!(tx.undone_at().is_none());
    }
  }

  mod latest_undoable_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_most_recent() {
      let (_store, conn, _tmp, pid) = setup().await;

      begin(&conn, &pid, "first").await.unwrap();
      let second = begin(&conn, &pid, "second").await.unwrap();

      let latest = latest_undoable(&conn, &pid).await.unwrap().unwrap();

      assert_eq!(latest.id(), second.id());
    }

    #[tokio::test]
    async fn it_returns_none_when_empty() {
      let (_store, conn, _tmp, pid) = setup().await;

      let latest = latest_undoable(&conn, &pid).await.unwrap();

      assert!(latest.is_none());
    }
  }

  mod latest_undoable_n_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_empty_when_none() {
      let (_store, conn, _tmp, pid) = setup().await;

      let results = latest_undoable_n(&conn, &pid, 3).await.unwrap();

      assert!(results.is_empty());
    }

    #[tokio::test]
    async fn it_returns_fewer_when_not_enough() {
      let (_store, conn, _tmp, pid) = setup().await;

      begin(&conn, &pid, "only one").await.unwrap();

      let results = latest_undoable_n(&conn, &pid, 5).await.unwrap();

      assert_eq!(results.len(), 1);
      assert_eq!(results[0].command(), "only one");
    }

    #[tokio::test]
    async fn it_returns_n_most_recent() {
      let (_store, conn, _tmp, pid) = setup().await;

      begin(&conn, &pid, "first").await.unwrap();
      begin(&conn, &pid, "second").await.unwrap();
      begin(&conn, &pid, "third").await.unwrap();

      let results = latest_undoable_n(&conn, &pid, 2).await.unwrap();

      assert_eq!(results.len(), 2);
      assert_eq!(results[0].command(), "third");
      assert_eq!(results[1].command(), "second");
    }
  }

  mod record_event_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    async fn semantic_row(conn: &Connection, tx_id: &Id) -> (Option<String>, Option<String>, Option<String>) {
      let mut rows = conn
        .query(
          "SELECT semantic_type, old_value, new_value FROM transaction_events WHERE transaction_id = ?1",
          [tx_id.to_string()],
        )
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      (row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap())
    }

    #[tokio::test]
    async fn it_leaves_semantic_fields_null_for_plain_record_event() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task update").await.unwrap();
      record_event(&conn, tx.id(), "tasks", "abc", "modified", None)
        .await
        .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic, None);
      assert_eq!(old, None);
      assert_eq!(new, None);
    }
  }

  mod record_semantic_event_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    async fn semantic_row(conn: &Connection, tx_id: &Id) -> (Option<String>, Option<String>, Option<String>) {
      let mut rows = conn
        .query(
          "SELECT semantic_type, old_value, new_value FROM transaction_events WHERE transaction_id = ?1",
          [tx_id.to_string()],
        )
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      (row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap())
    }

    #[tokio::test]
    async fn it_persists_archived_semantic_type() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "artifact archive").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "artifacts",
        "abc",
        "modified",
        None,
        Some("archived"),
        None,
        None,
      )
      .await
      .unwrap();

      let (semantic, _, _) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("archived"));
    }

    #[tokio::test]
    async fn it_persists_cancelled_semantic_type() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task cancel").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        "abc",
        "modified",
        None,
        Some("cancelled"),
        Some("open"),
        Some("cancelled"),
      )
      .await
      .unwrap();

      let (semantic, _, _) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("cancelled"));
    }

    #[tokio::test]
    async fn it_persists_completed_semantic_type() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task complete").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        "abc",
        "modified",
        None,
        Some("completed"),
        Some("open"),
        Some("done"),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("completed"));
      assert_eq!(old.as_deref(), Some("open"));
      assert_eq!(new.as_deref(), Some("done"));
    }

    #[tokio::test]
    async fn it_persists_created_semantic_type() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task create").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        "abc",
        "created",
        None,
        Some("created"),
        None,
        None,
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("created"));
      assert_eq!(old, None);
      assert_eq!(new, None);
    }

    #[tokio::test]
    async fn it_persists_phase_change_with_old_and_new_values() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task update").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "iteration_tasks",
        "abc",
        "modified",
        None,
        Some("phase-change"),
        Some("1"),
        Some("2"),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("phase-change"));
      assert_eq!(old.as_deref(), Some("1"));
      assert_eq!(new.as_deref(), Some("2"));
    }

    #[tokio::test]
    async fn it_persists_priority_change_with_old_and_new_values() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task update").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        "abc",
        "modified",
        None,
        Some("priority-change"),
        Some("2"),
        Some("1"),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("priority-change"));
      assert_eq!(old.as_deref(), Some("2"));
      assert_eq!(new.as_deref(), Some("1"));
    }

    #[tokio::test]
    async fn it_persists_status_change_with_old_and_new_values() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = begin(&conn, &pid, "task update").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        "abc",
        "modified",
        None,
        Some("status-change"),
        Some("open"),
        Some("in-progress"),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;

      assert_eq!(semantic.as_deref(), Some("status-change"));
      assert_eq!(old.as_deref(), Some("open"));
      assert_eq!(new.as_deref(), Some("in-progress"));
    }
  }

  mod undo_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_undoes_a_create_by_deleting() {
      let (_store, conn, _tmp, pid) = setup().await;

      // Create a task
      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [task_id.to_string(), pid.to_string(), "Undo me".to_string()],
        )
        .await
        .unwrap();

      // Record the transaction
      let tx = begin(&conn, &pid, "task create").await.unwrap();
      record_event(&conn, tx.id(), "tasks", &task_id.to_string(), "created", None)
        .await
        .unwrap();

      // Undo it
      let cmd = undo(&conn, tx.id()).await.unwrap();
      assert_eq!(cmd, "task create");

      // Task should be gone
      let mut rows = conn
        .query("SELECT id FROM tasks WHERE id = ?1", [task_id.to_string()])
        .await
        .unwrap();
      assert!(rows.next().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn it_undoes_a_semantic_modified_event_via_before_data() {
      let (_store, conn, _tmp, pid) = setup().await;

      // Insert a task in the `open` state
      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title, status) VALUES (?1, ?2, ?3, 'open')",
          [task_id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      // Capture before state, mutate, record semantic modified event
      let before = serde_json::json!({
        "id": task_id.to_string(),
        "status": "open",
        "title": "Task",
      });
      conn
        .execute(
          "UPDATE tasks SET status = 'in-progress' WHERE id = ?1",
          [task_id.to_string()],
        )
        .await
        .unwrap();

      let tx = begin(&conn, &pid, "task claim").await.unwrap();
      record_semantic_event(
        &conn,
        tx.id(),
        "tasks",
        &task_id.to_string(),
        "modified",
        Some(&before),
        Some("status-change"),
        Some("open"),
        Some("in-progress"),
      )
      .await
      .unwrap();

      // Undo should restore status to "open" from before_data, ignoring the semantic fields.
      undo(&conn, tx.id()).await.unwrap();

      let mut rows = conn
        .query("SELECT status FROM tasks WHERE id = ?1", [task_id.to_string()])
        .await
        .unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let status: String = row.get(0).unwrap();

      assert_eq!(status, "open");
    }
  }
}
