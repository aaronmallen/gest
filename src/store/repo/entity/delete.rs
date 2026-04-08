//! Cascade delete for top-level entities (artifacts, iterations, tasks).
//!
//! This helper performs a hard delete of an entity together with every
//! dependent row (notes, entity_tags, relationships, iteration_tasks),
//! recording a `deleted` event for each row via the transaction log so the
//! operation can be undone.
//!
//! Event ordering is intentional: children are recorded *before* the parent
//! so that the reverse-order undo replay re-inserts the parent row first and
//! every child row afterwards (whose FKs reference the parent). The forward
//! delete uses the same child-first ordering so FK constraints are never
//! violated during the mutation itself.
//!
//! This module does **not** write tombstone files, prompt the user, or
//! begin a transaction — those concerns belong to the caller.

#![allow(dead_code)]

use libsql::{Connection, Error as DbError};
use serde_json::{Map, Value as JsonValue};

use crate::store::{
  model::{Error as ModelError, primitives::EntityType},
  repo::transaction,
};

/// Errors returned by cascade delete operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A row could not be converted into a domain model.
  #[error(transparent)]
  Model(#[from] ModelError),
  /// An error from the transaction audit layer.
  #[error(transparent)]
  Transaction(#[from] transaction::Error),
}

/// Per-table counts of rows removed by a [`delete_with_cascade`] call.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeleteReport {
  pub iteration_tasks: usize,
  pub notes: usize,
  pub relationships: usize,
  pub tags: usize,
}

/// A single row captured prior to deletion, together with the metadata needed
/// to emit a transaction event.
struct CapturedRow {
  data: JsonValue,
  row_id: String,
  table: &'static str,
}

/// Delete an entity and all rows that depend on it, recording transaction
/// events for each removed row so the operation can be undone.
///
/// * `entity_id` must be the fully-resolved id of the entity.
/// * Callers are expected to have already begun a transaction and resolved
///   `transaction_id`; this helper only appends events and issues `DELETE`
///   statements.
///
/// Returns a [`DeleteReport`] with the number of rows removed from each
/// dependent table.
pub async fn delete_with_cascade(
  conn: &Connection,
  transaction_id: &crate::store::model::primitives::Id,
  entity_type: EntityType,
  entity_id: &crate::store::model::primitives::Id,
) -> Result<DeleteReport, Error> {
  log::debug!("repo::entity::delete::delete_with_cascade");
  let entity_type_str = entity_type.to_string();
  let id_str = entity_id.to_string();

  // Collect every dependent row first. We intentionally capture children in
  // the order they must be restored by undo: notes, entity_tags,
  // relationships, iteration_tasks — i.e. leaves before the joining table
  // that references the parent.
  let notes = capture_notes(conn, &entity_type_str, &id_str).await?;
  let entity_tags = capture_entity_tags(conn, &entity_type_str, &id_str).await?;
  let relationships = capture_relationships(conn, &entity_type_str, &id_str).await?;
  let iteration_tasks = match entity_type {
    EntityType::Task => capture_iteration_tasks_for_task(conn, &id_str).await?,
    EntityType::Iteration => capture_iteration_tasks_for_iteration(conn, &id_str).await?,
    EntityType::Artifact => Vec::new(),
  };
  let parent = capture_parent(conn, entity_type, &id_str).await?;

  let report = DeleteReport {
    iteration_tasks: iteration_tasks.len(),
    notes: notes.len(),
    relationships: relationships.len(),
    tags: entity_tags.len(),
  };

  // Record + delete children first, parent last. Within each child group the
  // record-then-delete pair keeps the audit log consistent with the physical
  // mutation.
  for row in notes
    .iter()
    .chain(entity_tags.iter())
    .chain(relationships.iter())
    .chain(iteration_tasks.iter())
  {
    record_and_delete_child(conn, transaction_id, row).await?;
  }

  record_and_delete_parent(conn, transaction_id, entity_type, &parent, &id_str).await?;

  Ok(report)
}

async fn capture_entity_tags(conn: &Connection, entity_type: &str, entity_id: &str) -> Result<Vec<CapturedRow>, Error> {
  let mut rows = conn
    .query(
      "SELECT entity_id, entity_type, tag_id, created_at \
        FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2",
      libsql::params![entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut captured = Vec::new();
  while let Some(row) = rows.next().await? {
    let entity_id_val: String = row.get(0)?;
    let entity_type_val: String = row.get(1)?;
    let tag_id: String = row.get(2)?;
    let created_at: String = row.get(3)?;

    let mut map = Map::new();
    map.insert("entity_id".into(), JsonValue::String(entity_id_val.clone()));
    map.insert("entity_type".into(), JsonValue::String(entity_type_val.clone()));
    map.insert("tag_id".into(), JsonValue::String(tag_id.clone()));
    map.insert("created_at".into(), JsonValue::String(created_at));

    captured.push(CapturedRow {
      data: JsonValue::Object(map),
      row_id: format!("{entity_type_val}:{entity_id_val}:{tag_id}"),
      table: "entity_tags",
    });
  }
  Ok(captured)
}

async fn capture_iteration_tasks_for_iteration(
  conn: &Connection,
  iteration_id: &str,
) -> Result<Vec<CapturedRow>, Error> {
  let mut rows = conn
    .query(
      "SELECT iteration_id, task_id, phase, created_at \
        FROM iteration_tasks WHERE iteration_id = ?1",
      [iteration_id.to_string()],
    )
    .await?;

  collect_iteration_task_rows(&mut rows).await
}

async fn capture_iteration_tasks_for_task(conn: &Connection, task_id: &str) -> Result<Vec<CapturedRow>, Error> {
  let mut rows = conn
    .query(
      "SELECT iteration_id, task_id, phase, created_at \
        FROM iteration_tasks WHERE task_id = ?1",
      [task_id.to_string()],
    )
    .await?;

  collect_iteration_task_rows(&mut rows).await
}

async fn capture_notes(conn: &Connection, entity_type: &str, entity_id: &str) -> Result<Vec<CapturedRow>, Error> {
  let mut rows = conn
    .query(
      "SELECT id, entity_id, entity_type, author_id, body, created_at, updated_at \
        FROM notes WHERE entity_type = ?1 AND entity_id = ?2",
      libsql::params![entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut captured = Vec::new();
  while let Some(row) = rows.next().await? {
    let id: String = row.get(0)?;
    let entity_id_val: String = row.get(1)?;
    let entity_type_val: String = row.get(2)?;
    let author_id: Option<String> = row.get(3)?;
    let body: String = row.get(4)?;
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;

    let mut map = Map::new();
    map.insert("id".into(), JsonValue::String(id.clone()));
    map.insert("entity_id".into(), JsonValue::String(entity_id_val));
    map.insert("entity_type".into(), JsonValue::String(entity_type_val));
    map.insert(
      "author_id".into(),
      match author_id {
        Some(a) => JsonValue::String(a),
        None => JsonValue::Null,
      },
    );
    map.insert("body".into(), JsonValue::String(body));
    map.insert("created_at".into(), JsonValue::String(created_at));
    map.insert("updated_at".into(), JsonValue::String(updated_at));

    captured.push(CapturedRow {
      data: JsonValue::Object(map),
      row_id: id,
      table: "notes",
    });
  }
  Ok(captured)
}

async fn capture_parent(conn: &Connection, entity_type: EntityType, id: &str) -> Result<JsonValue, Error> {
  let (sql, columns): (&str, &[&str]) = match entity_type {
    EntityType::Artifact => (
      "SELECT id, project_id, title, body, metadata, archived_at, created_at, updated_at \
        FROM artifacts WHERE id = ?1",
      &[
        "id",
        "project_id",
        "title",
        "body",
        "metadata",
        "archived_at",
        "created_at",
        "updated_at",
      ],
    ),
    EntityType::Iteration => (
      "SELECT id, project_id, title, status, description, metadata, completed_at, created_at, updated_at \
        FROM iterations WHERE id = ?1",
      &[
        "id",
        "project_id",
        "title",
        "status",
        "description",
        "metadata",
        "completed_at",
        "created_at",
        "updated_at",
      ],
    ),
    EntityType::Task => (
      "SELECT id, project_id, title, priority, status, description, assigned_to, metadata, \
        resolved_at, created_at, updated_at \
        FROM tasks WHERE id = ?1",
      &[
        "id",
        "project_id",
        "title",
        "priority",
        "status",
        "description",
        "assigned_to",
        "metadata",
        "resolved_at",
        "created_at",
        "updated_at",
      ],
    ),
  };

  let mut rows = conn.query(sql, [id.to_string()]).await?;
  let row = rows
    .next()
    .await?
    .ok_or_else(|| Error::Model(ModelError::InvalidValue(format!("{entity_type} not found: {id}"))))?;

  let mut map = Map::new();
  for (idx, column) in columns.iter().enumerate() {
    let value: libsql::Value = row.get(idx as i32)?;
    map.insert((*column).to_string(), libsql_value_to_json(value));
  }
  Ok(JsonValue::Object(map))
}

async fn capture_relationships(
  conn: &Connection,
  entity_type: &str,
  entity_id: &str,
) -> Result<Vec<CapturedRow>, Error> {
  let mut rows = conn
    .query(
      "SELECT id, rel_type, source_id, source_type, target_id, target_type, created_at, updated_at \
        FROM relationships \
        WHERE (source_type = ?1 AND source_id = ?2) OR (target_type = ?1 AND target_id = ?2)",
      libsql::params![entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut captured = Vec::new();
  while let Some(row) = rows.next().await? {
    let id: String = row.get(0)?;
    let rel_type: String = row.get(1)?;
    let source_id: String = row.get(2)?;
    let source_type: String = row.get(3)?;
    let target_id: String = row.get(4)?;
    let target_type: String = row.get(5)?;
    let created_at: String = row.get(6)?;
    let updated_at: String = row.get(7)?;

    let mut map = Map::new();
    map.insert("id".into(), JsonValue::String(id.clone()));
    map.insert("rel_type".into(), JsonValue::String(rel_type));
    map.insert("source_id".into(), JsonValue::String(source_id));
    map.insert("source_type".into(), JsonValue::String(source_type));
    map.insert("target_id".into(), JsonValue::String(target_id));
    map.insert("target_type".into(), JsonValue::String(target_type));
    map.insert("created_at".into(), JsonValue::String(created_at));
    map.insert("updated_at".into(), JsonValue::String(updated_at));

    captured.push(CapturedRow {
      data: JsonValue::Object(map),
      row_id: id,
      table: "relationships",
    });
  }
  Ok(captured)
}

async fn collect_iteration_task_rows(rows: &mut libsql::Rows) -> Result<Vec<CapturedRow>, Error> {
  let mut captured = Vec::new();
  while let Some(row) = rows.next().await? {
    let iteration_id: String = row.get(0)?;
    let task_id: String = row.get(1)?;
    let phase: i64 = row.get(2)?;
    let created_at: String = row.get(3)?;

    let mut map = Map::new();
    map.insert("iteration_id".into(), JsonValue::String(iteration_id.clone()));
    map.insert("task_id".into(), JsonValue::String(task_id.clone()));
    map.insert("phase".into(), JsonValue::Number(phase.into()));
    map.insert("created_at".into(), JsonValue::String(created_at));

    captured.push(CapturedRow {
      data: JsonValue::Object(map),
      row_id: format!("{iteration_id}:{task_id}"),
      table: "iteration_tasks",
    });
  }
  Ok(captured)
}

fn libsql_value_to_json(value: libsql::Value) -> JsonValue {
  match value {
    libsql::Value::Null => JsonValue::Null,
    libsql::Value::Integer(i) => JsonValue::Number(i.into()),
    libsql::Value::Real(f) => serde_json::Number::from_f64(f).map_or(JsonValue::Null, JsonValue::Number),
    libsql::Value::Text(s) => JsonValue::String(s),
    libsql::Value::Blob(b) => JsonValue::String(format!("<blob {} bytes>", b.len())),
  }
}

async fn record_and_delete_child(
  conn: &Connection,
  transaction_id: &crate::store::model::primitives::Id,
  row: &CapturedRow,
) -> Result<(), Error> {
  transaction::record_event(conn, transaction_id, row.table, &row.row_id, "deleted", Some(&row.data)).await?;

  match row.table {
    "entity_tags" => {
      let obj = row.data.as_object().expect("entity_tags row must be object");
      let entity_type = obj
        .get("entity_type")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
      let entity_id = obj
        .get("entity_id")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
      let tag_id = obj
        .get("tag_id")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
      conn
        .execute(
          "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2 AND tag_id = ?3",
          libsql::params![entity_type, entity_id, tag_id],
        )
        .await?;
    }
    "iteration_tasks" => {
      let obj = row.data.as_object().expect("iteration_tasks row must be object");
      let iteration_id = obj
        .get("iteration_id")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
      let task_id = obj
        .get("task_id")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .to_string();
      conn
        .execute(
          "DELETE FROM iteration_tasks WHERE iteration_id = ?1 AND task_id = ?2",
          libsql::params![iteration_id, task_id],
        )
        .await?;
    }
    "notes" => {
      conn
        .execute("DELETE FROM notes WHERE id = ?1", [row.row_id.clone()])
        .await?;
    }
    "relationships" => {
      conn
        .execute("DELETE FROM relationships WHERE id = ?1", [row.row_id.clone()])
        .await?;
    }
    other => {
      return Err(Error::Model(ModelError::InvalidValue(format!(
        "unexpected child table in cascade delete: {other}"
      ))));
    }
  }
  Ok(())
}

async fn record_and_delete_parent(
  conn: &Connection,
  transaction_id: &crate::store::model::primitives::Id,
  entity_type: EntityType,
  before: &JsonValue,
  id: &str,
) -> Result<(), Error> {
  let (table, sql) = match entity_type {
    EntityType::Artifact => ("artifacts", "DELETE FROM artifacts WHERE id = ?1"),
    EntityType::Iteration => ("iterations", "DELETE FROM iterations WHERE id = ?1"),
    EntityType::Task => ("tasks", "DELETE FROM tasks WHERE id = ?1"),
  };

  transaction::record_event(conn, transaction_id, table, id, "deleted", Some(before)).await?;
  conn.execute(sql, [id.to_string()]).await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use tempfile::TempDir;

  use super::*;
  use crate::store::{
    self, Db,
    model::{
      Project,
      primitives::{Id, RelationshipType},
    },
    repo::{
      artifact as artifact_repo, iteration as iteration_repo, note as note_repo, relationship as relationship_repo,
      tag as tag_repo, task as task_repo, transaction as transaction_repo,
    },
  };

  async fn setup() -> (Arc<Db>, libsql::Connection, TempDir, Id) {
    let (store, tmp) = store::open_temp().await.unwrap();
    let conn = store.connect().await.unwrap();
    let project = Project::new("/tmp/delete-test".into());
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

  async fn row_count(conn: &libsql::Connection, sql: &str, params: &[libsql::Value]) -> i64 {
    let mut rows = conn.query(sql, params.to_vec()).await.unwrap();
    let row = rows.next().await.unwrap().unwrap();
    row.get(0).unwrap()
  }

  mod delete_with_cascade_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_cascades_a_task_with_notes_tags_relationships_and_iteration_tasks() {
      let (_store, conn, _tmp, pid) = setup().await;

      let task = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "Target".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let other_task = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "Other".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let iteration = iteration_repo::create(
        &conn,
        &pid,
        &crate::store::model::iteration::New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      iteration_repo::add_task(&conn, iteration.id(), task.id(), 1)
        .await
        .unwrap();

      note_repo::create(
        &conn,
        EntityType::Task,
        task.id(),
        &crate::store::model::note::New {
          body: "note body".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();
      tag_repo::attach(&conn, EntityType::Task, task.id(), "blocker")
        .await
        .unwrap();
      relationship_repo::create(
        &conn,
        RelationshipType::BlockedBy,
        EntityType::Task,
        task.id(),
        EntityType::Task,
        other_task.id(),
      )
      .await
      .unwrap();

      let tx = transaction_repo::begin(&conn, &pid, "task delete").await.unwrap();
      let report = delete_with_cascade(&conn, tx.id(), EntityType::Task, task.id())
        .await
        .unwrap();

      assert_eq!(report.notes, 1);
      assert_eq!(report.tags, 1);
      assert_eq!(report.relationships, 1);
      assert_eq!(report.iteration_tasks, 1);

      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM tasks WHERE id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM notes WHERE entity_type = 'task' AND entity_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'task' AND entity_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM relationships WHERE source_id = ?1 OR target_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM iteration_tasks WHERE task_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        0
      );
    }

    #[tokio::test]
    async fn it_cascades_an_artifact_with_notes_tags_and_relationships() {
      let (_store, conn, _tmp, pid) = setup().await;

      let artifact = artifact_repo::create(
        &conn,
        &pid,
        &crate::store::model::artifact::New {
          title: "Spec".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let task = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "Related".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();

      note_repo::create(
        &conn,
        EntityType::Artifact,
        artifact.id(),
        &crate::store::model::note::New {
          body: "spec note".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();
      tag_repo::attach(&conn, EntityType::Artifact, artifact.id(), "design")
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

      let tx = transaction_repo::begin(&conn, &pid, "artifact delete").await.unwrap();
      let report = delete_with_cascade(&conn, tx.id(), EntityType::Artifact, artifact.id())
        .await
        .unwrap();

      assert_eq!(report.notes, 1);
      assert_eq!(report.tags, 1);
      assert_eq!(report.relationships, 1);
      assert_eq!(report.iteration_tasks, 0);

      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM artifacts WHERE id = ?1",
          &[artifact.id().to_string().into()],
        )
        .await,
        0
      );
    }

    #[tokio::test]
    async fn it_cascades_an_iteration_with_iteration_tasks() {
      let (_store, conn, _tmp, pid) = setup().await;

      let iteration = iteration_repo::create(
        &conn,
        &pid,
        &crate::store::model::iteration::New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let task_a = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "A".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let task_b = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "B".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      iteration_repo::add_task(&conn, iteration.id(), task_a.id(), 1)
        .await
        .unwrap();
      iteration_repo::add_task(&conn, iteration.id(), task_b.id(), 2)
        .await
        .unwrap();

      let tx = transaction_repo::begin(&conn, &pid, "iteration delete").await.unwrap();
      let report = delete_with_cascade(&conn, tx.id(), EntityType::Iteration, iteration.id())
        .await
        .unwrap();

      assert_eq!(report.iteration_tasks, 2);
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM iterations WHERE id = ?1",
          &[iteration.id().to_string().into()],
        )
        .await,
        0
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM iteration_tasks WHERE iteration_id = ?1",
          &[iteration.id().to_string().into()],
        )
        .await,
        0
      );
      // Sibling tasks must not be touched.
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM tasks WHERE id = ?1",
          &[task_a.id().to_string().into()]
        )
        .await,
        1
      );
    }

    #[tokio::test]
    async fn it_is_reversible_via_transaction_undo() {
      let (_store, conn, _tmp, pid) = setup().await;

      let task = task_repo::create(
        &conn,
        &pid,
        &crate::store::model::task::New {
          title: "Reversible".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      let iteration = iteration_repo::create(
        &conn,
        &pid,
        &crate::store::model::iteration::New {
          title: "Sprint".into(),
          ..Default::default()
        },
      )
      .await
      .unwrap();
      iteration_repo::add_task(&conn, iteration.id(), task.id(), 3)
        .await
        .unwrap();

      note_repo::create(
        &conn,
        EntityType::Task,
        task.id(),
        &crate::store::model::note::New {
          body: "need".into(),
          author_id: None,
        },
      )
      .await
      .unwrap();
      tag_repo::attach(&conn, EntityType::Task, task.id(), "urgent")
        .await
        .unwrap();

      let tx = transaction_repo::begin(&conn, &pid, "task delete").await.unwrap();
      delete_with_cascade(&conn, tx.id(), EntityType::Task, task.id())
        .await
        .unwrap();

      transaction_repo::undo(&conn, tx.id()).await.unwrap();

      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM tasks WHERE id = ?1",
          &[task.id().to_string().into()]
        )
        .await,
        1
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM notes WHERE entity_type = 'task' AND entity_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        1
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'task' AND entity_id = ?1",
          &[task.id().to_string().into()],
        )
        .await,
        1
      );
      assert_eq!(
        row_count(
          &conn,
          "SELECT COUNT(*) FROM iteration_tasks WHERE iteration_id = ?1 AND task_id = ?2",
          &[iteration.id().to_string().into(), task.id().to_string().into()],
        )
        .await,
        1
      );
    }
  }
}
