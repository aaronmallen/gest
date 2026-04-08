use chrono::Utc;
use libsql::{Connection, Error as DbError, Value};

use crate::{
  store::model::{
    Error as ModelError,
    iteration::{Filter, Model, New, Patch},
    primitives::{Id, IterationStatus},
  },
  ui::components::min_unique_prefix,
};

/// Errors that can occur in iteration repository operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// The requested entity was not found.
  #[error("iteration not found: {0}")]
  NotFound(String),
}

const SELECT_COLUMNS: &str = "\
  id, project_id, title, status, description, \
  metadata, completed_at, created_at, updated_at";

/// A task's summary info within an iteration context.
pub struct IterationTaskRow {
  /// Truncated task id suitable for display.
  pub id_short: String,
  /// Phase number within the iteration.
  pub phase: u32,
  /// Task status string as stored in the database.
  pub status: String,
  /// Task title.
  pub title: String,
}

/// Status counts for tasks in an iteration.
pub struct StatusCounts {
  /// Number of tasks in the `cancelled` state.
  pub cancelled: i64,
  /// Number of tasks in the `done` state.
  pub done: i64,
  /// Number of tasks in the `in-progress` state.
  pub in_progress: i64,
  /// Number of tasks in the `open` state.
  pub open: i64,
  /// Total number of tasks across all states.
  pub total: i64,
}

/// Add a task to an iteration at a specific phase.
pub async fn add_task(conn: &Connection, iteration_id: &Id, task_id: &Id, phase: u32) -> Result<(), Error> {
  log::debug!("repo::iteration::add_task");
  conn
    .execute(
      "INSERT OR IGNORE INTO iteration_tasks (iteration_id, task_id, phase, created_at) VALUES (?1, ?2, ?3, ?4)",
      libsql::params![
        iteration_id.to_string(),
        task_id.to_string(),
        phase as i64,
        Utc::now().to_rfc3339(),
      ],
    )
    .await?;
  Ok(())
}

/// Return iterations for a project, applying the given filter.
pub async fn all(conn: &Connection, project_id: &Id, filter: &Filter) -> Result<Vec<Model>, Error> {
  log::debug!("repo::iteration::all");
  let mut conditions = vec!["project_id = ?1".to_string()];
  let mut params: Vec<Value> = vec![Value::from(project_id.to_string())];
  let mut idx = 2;

  if !filter.all {
    conditions.push("status NOT IN ('completed', 'cancelled')".to_string());
  }

  if let Some(status) = &filter.status {
    conditions.push(format!("status = ?{idx}"));
    params.push(Value::from(status.to_string()));
    idx += 1;
  }

  if filter.has_available {
    conditions.push(
      "id IN (SELECT it.iteration_id FROM iteration_tasks it \
        INNER JOIN tasks t ON t.id = it.task_id \
        WHERE t.status = 'open')"
        .to_string(),
    );
  }

  if let Some(tag) = &filter.tag {
    conditions.push(format!(
      "id IN (SELECT et.entity_id FROM entity_tags et \
        INNER JOIN tags t ON t.id = et.tag_id \
        WHERE et.entity_type = 'iteration' AND t.label = ?{idx})"
    ));
    params.push(Value::from(tag.clone()));
    let _ = idx;
  }

  let where_clause = conditions.join(" AND ");
  let sql = format!("SELECT {SELECT_COLUMNS} FROM iterations WHERE {where_clause} ORDER BY created_at DESC");

  let mut rows = conn.query(&sql, libsql::params_from_iter(params)).await?;
  let mut iterations = Vec::new();
  while let Some(row) = rows.next().await? {
    iterations.push(Model::try_from(row)?);
  }
  Ok(iterations)
}

/// Create a new iteration in the given project.
pub async fn create(conn: &Connection, project_id: &Id, new: &New) -> Result<Model, Error> {
  log::debug!("repo::iteration::create");
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
        "INSERT INTO iterations ({SELECT_COLUMNS}) \
          VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8)"
      ),
      libsql::params![
        id.to_string(),
        project_id.to_string(),
        new.title.clone(),
        IterationStatus::default().to_string(),
        new.description.clone(),
        metadata,
        now.to_rfc3339(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("iteration not found after insert".into())))
}

/// Find an iteration by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  log::debug!("repo::iteration::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM iterations WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Get the maximum phase number for an iteration, or None if empty.
pub async fn max_phase(conn: &Connection, iteration_id: &Id) -> Result<Option<u32>, Error> {
  log::debug!("repo::iteration::max_phase");
  let mut rows = conn
    .query(
      "SELECT MAX(phase) FROM iteration_tasks WHERE iteration_id = ?1",
      [iteration_id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => {
      let max: Option<i64> = row.get(0)?;
      Ok(max.map(|m| m as u32))
    }
    None => Ok(None),
  }
}

/// Remove a task from an iteration.
pub async fn remove_task(conn: &Connection, iteration_id: &Id, task_id: &Id) -> Result<bool, Error> {
  log::debug!("repo::iteration::remove_task");
  let affected = conn
    .execute(
      "DELETE FROM iteration_tasks WHERE iteration_id = ?1 AND task_id = ?2",
      [iteration_id.to_string(), task_id.to_string()],
    )
    .await?;
  Ok(affected > 0)
}

/// Return task counts grouped by status for an iteration.
pub async fn task_status_counts(conn: &Connection, iteration_id: &Id) -> Result<StatusCounts, Error> {
  log::debug!("repo::iteration::task_status_counts");
  let mut rows = conn
    .query(
      "SELECT t.status, COUNT(*) FROM iteration_tasks it \
        JOIN tasks t ON t.id = it.task_id \
        WHERE it.iteration_id = ?1 GROUP BY t.status",
      [iteration_id.to_string()],
    )
    .await?;

  let mut counts = StatusCounts {
    cancelled: 0,
    done: 0,
    in_progress: 0,
    open: 0,
    total: 0,
  };
  while let Some(row) = rows.next().await? {
    let status: String = row.get(0)?;
    let count: i64 = row.get(1)?;
    counts.total += count;
    match status.as_str() {
      "cancelled" => counts.cancelled = count,
      "done" => counts.done = count,
      "in-progress" => counts.in_progress = count,
      "open" => counts.open = count,
      _ => {}
    }
  }
  Ok(counts)
}

/// Return all tasks for an iteration, grouped with their phase and status.
pub async fn tasks_with_phase(conn: &Connection, iteration_id: &Id) -> Result<Vec<IterationTaskRow>, Error> {
  log::debug!("repo::iteration::tasks_with_phase");
  let mut rows = conn
    .query(
      "SELECT t.id, t.title, t.status, it.phase FROM iteration_tasks it \
        JOIN tasks t ON t.id = it.task_id \
        WHERE it.iteration_id = ?1 ORDER BY it.phase, t.title",
      [iteration_id.to_string()],
    )
    .await?;

  let mut result = Vec::new();
  while let Some(row) = rows.next().await? {
    let full_id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let status: String = row.get(2)?;
    let phase: i64 = row.get(3)?;
    let id_short = if full_id.len() >= 8 {
      full_id[..8].to_string()
    } else {
      full_id
    };
    result.push(IterationTaskRow {
      id_short,
      phase: phase as u32,
      status,
      title,
    });
  }
  Ok(result)
}

/// Return the minimum unique prefix length over all active iterations (status
/// not `completed` or `cancelled`) in the project.
pub async fn shortest_active_prefix(conn: &Connection, project_id: &Id) -> Result<usize, Error> {
  log::debug!("repo::iteration::shortest_active_prefix");
  let ids = collect_ids(
    conn,
    "SELECT id FROM iterations WHERE project_id = ?1 AND status NOT IN ('completed', 'cancelled')",
    project_id,
  )
  .await?;
  let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
  Ok(min_unique_prefix(&refs))
}

/// Return the minimum unique prefix length over every iteration in the project.
pub async fn shortest_all_prefix(conn: &Connection, project_id: &Id) -> Result<usize, Error> {
  log::debug!("repo::iteration::shortest_all_prefix");
  let ids = collect_ids(conn, "SELECT id FROM iterations WHERE project_id = ?1", project_id).await?;
  let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
  Ok(min_unique_prefix(&refs))
}

async fn collect_ids(conn: &Connection, sql: &str, project_id: &Id) -> Result<Vec<String>, Error> {
  let mut rows = conn.query(sql, [project_id.to_string()]).await?;
  let mut ids = Vec::new();
  while let Some(row) = rows.next().await? {
    ids.push(row.get::<String>(0)?);
  }
  Ok(ids)
}

/// Update an existing iteration with the given patch.
pub async fn update(conn: &Connection, id: &Id, patch: &Patch) -> Result<Model, Error> {
  log::debug!("repo::iteration::update");
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
      sets.push(format!("completed_at = ?{idx}"));
      params.push(Value::from(now.to_rfc3339()));
      idx += 1;
    } else {
      sets.push("completed_at = NULL".to_string());
    }
  }

  if let Some(metadata) = &patch.metadata {
    sets.push(format!("metadata = ?{idx}"));
    params.push(Value::from(metadata.to_string()));
    idx += 1;
  }

  let set_clause = sets.join(", ");
  params.push(Value::from(id.to_string()));
  let sql = format!("UPDATE iterations SET {set_clause} WHERE id = ?{idx}");

  let affected = conn.execute(&sql, libsql::params_from_iter(params)).await?;

  if affected == 0 {
    return Err(Error::NotFound(id.short()));
  }

  find_by_id(conn, id.clone())
    .await?
    .ok_or_else(|| Error::NotFound(id.short()))
}

/// Return the current phase of a task within its iteration, if any.
pub async fn task_phase(conn: &Connection, task_id: &Id) -> Result<Option<u32>, Error> {
  log::debug!("repo::iteration::task_phase");
  let mut rows = conn
    .query(
      "SELECT phase FROM iteration_tasks WHERE task_id = ?1",
      [task_id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => {
      let phase: i64 = row.get(0)?;
      Ok(Some(phase as u32))
    }
    None => Ok(None),
  }
}

/// Update the phase of a task within its iteration.
pub async fn update_task_phase(conn: &Connection, task_id: &Id, phase: u32) -> Result<(), Error> {
  log::debug!("repo::iteration::update_task_phase");
  conn
    .execute(
      "UPDATE iteration_tasks SET phase = ?1 WHERE task_id = ?2",
      libsql::params![phase as i64, task_id.to_string()],
    )
    .await?;
  Ok(())
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
    let project = Project::new("/tmp/iteration-test".into());
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

  mod add_task_fn {
    use super::*;

    #[tokio::test]
    async fn it_adds_a_task_to_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;
      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Iter".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [task_id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      add_task(&conn, iter.id(), &task_id, 1).await.unwrap();

      let max = max_phase(&conn, iter.id()).await.unwrap();
      assert_eq!(max, Some(1));
    }
  }

  mod all_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_excludes_iterations_with_only_claimed_tasks() {
      let (_store, conn, _tmp, pid) = setup().await;

      let iter = create(
        &conn,
        &pid,
        &New {
          title: "All claimed".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title, status) VALUES (?1, ?2, ?3, 'in-progress')",
          [task_id.to_string(), pid.to_string(), "Claimed task".to_string()],
        )
        .await
        .unwrap();
      add_task(&conn, iter.id(), &task_id, 0).await.unwrap();

      let filter = Filter {
        has_available: true,
        ..Default::default()
      };
      let results = all(&conn, &pid, &filter).await.unwrap();

      assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn it_filters_by_has_available() {
      let (_store, conn, _tmp, pid) = setup().await;

      let with_open = create(
        &conn,
        &pid,
        &New {
          title: "With open".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let _without_open = create(
        &conn,
        &pid,
        &New {
          title: "Without open".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title, status) VALUES (?1, ?2, ?3, 'open')",
          [task_id.to_string(), pid.to_string(), "Open task".to_string()],
        )
        .await
        .unwrap();
      add_task(&conn, with_open.id(), &task_id, 0).await.unwrap();

      let filter = Filter {
        has_available: true,
        ..Default::default()
      };
      let results = all(&conn, &pid, &filter).await.unwrap();

      assert_eq!(results.len(), 1);
      assert_eq!(results[0].title(), "With open");
    }
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;

      let new = New {
        description: "Sprint 1".into(),
        title: "Sprint 1".into(),
        ..Default::default()
      };
      let iteration = create(&conn, &pid, &new).await.unwrap();

      assert_eq!(iteration.title(), "Sprint 1");
      assert_eq!(iteration.status(), IterationStatus::Active);
      assert!(iteration.completed_at().is_none());
    }
  }

  mod remove_task_fn {
    use super::*;

    #[tokio::test]
    async fn it_removes_a_task_from_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;
      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Iter".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [task_id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      add_task(&conn, iter.id(), &task_id, 1).await.unwrap();
      let removed = remove_task(&conn, iter.id(), &task_id).await.unwrap();
      assert!(removed);

      let max = max_phase(&conn, iter.id()).await.unwrap();
      assert_eq!(max, None);
    }
  }

  mod semantic_events {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::store::repo::transaction;

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
    async fn it_records_a_completed_event_when_completing() {
      let (_store, conn, _tmp, pid) = setup().await;

      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let before = serde_json::to_value(&iter).unwrap();

      let tx = transaction::begin(&conn, &pid, "iteration complete").await.unwrap();
      let updated = update(
        &conn,
        iter.id(),
        &Patch {
          status: Some(IterationStatus::Completed),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      transaction::record_semantic_event(
        &conn,
        tx.id(),
        "iterations",
        &iter.id().to_string(),
        "modified",
        Some(&before),
        Some("completed"),
        Some(&iter.status().to_string()),
        Some(&updated.status().to_string()),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;
      assert_eq!(semantic.as_deref(), Some("completed"));
      assert_eq!(old.as_deref(), Some("active"));
      assert_eq!(new.as_deref(), Some("completed"));
    }

    #[tokio::test]
    async fn it_records_a_created_event_when_creating_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;

      let tx = transaction::begin(&conn, &pid, "iteration create").await.unwrap();
      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      transaction::record_semantic_event(
        &conn,
        tx.id(),
        "iterations",
        &iter.id().to_string(),
        "created",
        None,
        Some("created"),
        None,
        None,
      )
      .await
      .unwrap();

      let (semantic, _, _) = semantic_row(&conn, tx.id()).await;
      assert_eq!(semantic.as_deref(), Some("created"));
    }

    #[tokio::test]
    async fn it_records_a_phase_change_event_when_updating_task_phase() {
      let (_store, conn, _tmp, pid) = setup().await;

      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let task_id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [task_id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();
      add_task(&conn, iter.id(), &task_id, 1).await.unwrap();

      let old = task_phase(&conn, &task_id).await.unwrap().unwrap();
      let tx = transaction::begin(&conn, &pid, "task update").await.unwrap();
      update_task_phase(&conn, &task_id, 2).await.unwrap();
      transaction::record_semantic_event(
        &conn,
        tx.id(),
        "iteration_tasks",
        &task_id.to_string(),
        "modified",
        None,
        Some("phase-change"),
        Some(&old.to_string()),
        Some("2"),
      )
      .await
      .unwrap();

      let (semantic, old, new) = semantic_row(&conn, tx.id()).await;
      assert_eq!(semantic.as_deref(), Some("phase-change"));
      assert_eq!(old.as_deref(), Some("1"));
      assert_eq!(new.as_deref(), Some("2"));
    }
  }

  mod shortest_prefix_fns {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::ui::components::min_unique_prefix;

    #[tokio::test]
    async fn it_matches_min_unique_prefix_over_active_iterations() {
      let (_store, conn, _tmp, pid) = setup().await;

      let mut active_ids = Vec::new();
      for i in 0..4 {
        let iter = create(
          &conn,
          &pid,
          &New {
            title: format!("Iter {i}"),
            ..Default::default()
          },
        )
        .await
        .unwrap();
        active_ids.push(iter.id().to_string());
      }
      let done = create(
        &conn,
        &pid,
        &New {
          title: "Done iter".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      update(
        &conn,
        done.id(),
        &Patch {
          status: Some(IterationStatus::Completed),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let refs: Vec<&str> = active_ids.iter().map(String::as_str).collect();
      let expected = min_unique_prefix(&refs);
      let got = shortest_active_prefix(&conn, &pid).await.unwrap();

      assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn it_matches_min_unique_prefix_over_all_iterations() {
      let (_store, conn, _tmp, pid) = setup().await;

      let mut all_ids = Vec::new();
      for i in 0..3 {
        let iter = create(
          &conn,
          &pid,
          &New {
            title: format!("Iter {i}"),
            ..Default::default()
          },
        )
        .await
        .unwrap();
        all_ids.push(iter.id().to_string());
      }
      let done = create(
        &conn,
        &pid,
        &New {
          title: "Done iter".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      all_ids.push(done.id().to_string());
      update(
        &conn,
        done.id(),
        &Patch {
          status: Some(IterationStatus::Completed),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let refs: Vec<&str> = all_ids.iter().map(String::as_str).collect();
      let expected = min_unique_prefix(&refs);
      let got = shortest_all_prefix(&conn, &pid).await.unwrap();

      assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn it_returns_one_for_empty_population() {
      let (_store, conn, _tmp, pid) = setup().await;

      assert_eq!(shortest_active_prefix(&conn, &pid).await.unwrap(), 1);
      assert_eq!(shortest_all_prefix(&conn, &pid).await.unwrap(), 1);
    }
  }

  mod update_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_completes_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;
      let iter = create(
        &conn,
        &pid,
        &New {
          title: "Iter".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      let updated = update(
        &conn,
        iter.id(),
        &Patch {
          status: Some(IterationStatus::Completed),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      assert_eq!(updated.status(), IterationStatus::Completed);
      assert!(updated.completed_at().is_some());
    }
  }
}
