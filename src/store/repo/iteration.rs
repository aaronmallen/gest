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
  id, project_id, completed_at, created_at, description, \
  metadata, status, title, updated_at";

/// A task's summary info within an iteration context.
pub struct IterationTaskRow {
  pub id_short: String,
  pub phase: u32,
  pub status: String,
  pub title: String,
}

/// Status counts for tasks in an iteration.
pub struct StatusCounts {
  pub cancelled: i64,
  pub done: i64,
  pub in_progress: i64,
  pub open: i64,
  pub total: i64,
}

/// Add a task to an iteration at a specific phase.
pub async fn add_task(conn: &Connection, iteration_id: &Id, task_id: &Id, phase: u32) -> Result<(), Error> {
  conn
    .execute(
      "INSERT OR IGNORE INTO iteration_tasks (iteration_id, task_id, phase) VALUES (?1, ?2, ?3)",
      libsql::params![iteration_id.to_string(), task_id.to_string(), phase as i64],
    )
    .await?;
  Ok(())
}

/// Return iterations for a project, applying the given filter.
pub async fn all(conn: &Connection, project_id: &Id, filter: &Filter) -> Result<Vec<Model>, Error> {
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
          VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8)"
      ),
      libsql::params![
        id.to_string(),
        project_id.to_string(),
        now.to_rfc3339(),
        new.description.clone(),
        metadata,
        IterationStatus::default().to_string(),
        new.title.clone(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue("iteration not found after insert".into())))
}

/// Delete an iteration by its ID. Returns true if the iteration was deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  // Remove task associations first
  conn
    .execute("DELETE FROM iteration_tasks WHERE iteration_id = ?1", [id.to_string()])
    .await?;
  let affected = conn
    .execute("DELETE FROM iterations WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find an iteration by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
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

/// Update the phase of a task within its iteration.
pub async fn update_task_phase(conn: &Connection, task_id: &Id, phase: u32) -> Result<(), Error> {
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

  mod all_fn {
    use pretty_assertions::assert_eq;

    use super::*;

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
}
