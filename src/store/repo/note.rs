use chrono::Utc;
use libsql::{Connection, Value};

use crate::store::{
  Error,
  model::{
    note::{Model, New, Patch},
    primitives::{EntityType, Id},
  },
};

const SELECT_COLUMNS: &str = "id, entity_id, entity_type, author_id, body, created_at, updated_at";

/// Create a new note on an entity.
pub async fn create(conn: &Connection, entity_type: EntityType, entity_id: &Id, new: &New) -> Result<Model, Error> {
  log::debug!("repo::note::create");
  let id = Id::new();
  let now = Utc::now();
  let author_id: Value = match &new.author_id {
    Some(a) => Value::from(a.to_string()),
    None => Value::Null,
  };

  conn
    .execute(
      &format!("INSERT INTO notes ({SELECT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
      libsql::params![
        id.to_string(),
        entity_id.to_string(),
        entity_type.to_string(),
        author_id,
        new.body.clone(),
        now.to_rfc3339(),
        now.to_rfc3339(),
      ],
    )
    .await?;

  find_by_id(conn, id)
    .await?
    .ok_or_else(|| Error::InvalidValue("note not found after insert".into()))
}

/// Delete a note by its ID. Returns true if the note was deleted.
pub async fn delete(conn: &Connection, id: &Id) -> Result<bool, Error> {
  log::debug!("repo::note::delete");
  let affected = conn
    .execute("DELETE FROM notes WHERE id = ?1", [id.to_string()])
    .await?;
  Ok(affected > 0)
}

/// Find a note by its [`Id`].
pub async fn find_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Option<Model>, Error> {
  log::debug!("repo::note::find_by_id");
  let id = id.into();
  let mut rows = conn
    .query(
      &format!("SELECT {SELECT_COLUMNS} FROM notes WHERE id = ?1"),
      [id.to_string()],
    )
    .await?;

  match rows.next().await? {
    Some(row) => Ok(Some(Model::try_from(row)?)),
    None => Ok(None),
  }
}

/// Find a note by its [`Id`], returning [`Error::NotFound`] when no row matches.
pub async fn find_required_by_id(conn: &Connection, id: impl Into<Id>) -> Result<Model, Error> {
  let id = id.into();
  find_by_id(conn, id.clone())
    .await?
    .ok_or_else(|| Error::NotFound(format!("note {}", id.short())))
}

/// Return all notes for a specific entity, newest first.
pub async fn for_entity(conn: &Connection, entity_type: EntityType, entity_id: &Id) -> Result<Vec<Model>, Error> {
  log::debug!("repo::note::for_entity");
  let mut rows = conn
    .query(
      &format!(
        "SELECT {SELECT_COLUMNS} FROM notes \
          WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY created_at DESC"
      ),
      [entity_type.to_string(), entity_id.to_string()],
    )
    .await?;

  let mut notes = Vec::new();
  while let Some(row) = rows.next().await? {
    notes.push(Model::try_from(row)?);
  }
  Ok(notes)
}

/// Return the minimum unique prefix length over all notes attached to the
/// given parent artifact.
#[cfg(test)]
pub async fn shortest_prefix(conn: &Connection, artifact_id: &Id) -> Result<usize, Error> {
  log::debug!("repo::note::shortest_prefix");
  let mut rows = conn
    .query(
      "SELECT id FROM notes WHERE entity_type = 'artifact' AND entity_id = ?1",
      [artifact_id.to_string()],
    )
    .await?;
  let mut ids = Vec::new();
  while let Some(row) = rows.next().await? {
    ids.push(row.get::<String>(0)?);
  }
  let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
  Ok(crate::ui::components::min_unique_prefix(&refs))
}

/// Update an existing note with the given patch.
pub async fn update(conn: &Connection, id: &Id, patch: &Patch) -> Result<Model, Error> {
  log::debug!("repo::note::update");
  let now = Utc::now();
  let mut sets = vec!["updated_at = ?1".to_string()];
  let mut params: Vec<Value> = vec![Value::from(now.to_rfc3339())];
  let mut idx = 2;

  if let Some(body) = &patch.body {
    sets.push(format!("body = ?{idx}"));
    params.push(Value::from(body.clone()));
    idx += 1;
  }

  let set_clause = sets.join(", ");
  params.push(Value::from(id.to_string()));
  let sql = format!("UPDATE notes SET {set_clause} WHERE id = ?{idx}");

  let affected = conn.execute(&sql, libsql::params_from_iter(params)).await?;

  if affected == 0 {
    return Err(Error::NotFound(format!("note {}", id.short())));
  }

  find_by_id(conn, id.clone())
    .await?
    .ok_or_else(|| Error::NotFound(format!("note {}", id.short())))
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
    let project = Project::new("/tmp/note-test".into());
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
    let task_id = Id::new();
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
        [task_id.to_string(), pid.to_string(), "Test task".to_string()],
      )
      .await
      .unwrap();
    (store, conn, tmp, pid, task_id)
  }

  mod create_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_creates_a_note() {
      let (_store, conn, _tmp, _pid, task_id) = setup().await;

      let new = New {
        author_id: None,
        body: "A note".into(),
      };
      let note = create(&conn, EntityType::Task, &task_id, &new).await.unwrap();

      assert_eq!(note.body(), "A note");
      assert_eq!(note.entity_type(), EntityType::Task);
    }
  }

  mod delete_fn {
    use super::*;

    #[tokio::test]
    async fn it_deletes_a_note() {
      let (_store, conn, _tmp, _pid, task_id) = setup().await;
      let note = create(
        &conn,
        EntityType::Task,
        &task_id,
        &New {
          author_id: None,
          body: "Delete".into(),
        },
      )
      .await
      .unwrap();

      let deleted = delete(&conn, note.id()).await.unwrap();

      assert!(deleted);

      let found = find_by_id(&conn, note.id().clone()).await.unwrap();
      assert!(found.is_none());
    }
  }

  mod for_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_returns_notes_for_entity() {
      let (_store, conn, _tmp, _pid, task_id) = setup().await;

      create(
        &conn,
        EntityType::Task,
        &task_id,
        &New {
          author_id: None,
          body: "First".into(),
        },
      )
      .await
      .unwrap();
      create(
        &conn,
        EntityType::Task,
        &task_id,
        &New {
          author_id: None,
          body: "Second".into(),
        },
      )
      .await
      .unwrap();

      let notes = for_entity(&conn, EntityType::Task, &task_id).await.unwrap();

      assert_eq!(notes.len(), 2);
    }
  }

  mod shortest_prefix_fn {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::ui::components::min_unique_prefix;

    async fn make_artifact(conn: &Connection, project_id: &Id) -> Id {
      let artifact_id = Id::new();
      conn
        .execute(
          "INSERT INTO artifacts (id, project_id, title, body, created_at, updated_at, metadata) \
            VALUES (?1, ?2, 'A', '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z', '{}')",
          [artifact_id.to_string(), project_id.to_string()],
        )
        .await
        .unwrap();
      artifact_id
    }

    #[tokio::test]
    async fn it_matches_min_unique_prefix_over_artifact_notes() {
      let (_store, conn, _tmp, pid, _task_id) = setup().await;
      let artifact_id = make_artifact(&conn, &pid).await;

      let mut ids = Vec::new();
      for i in 0..5 {
        let note = create(
          &conn,
          EntityType::Artifact,
          &artifact_id,
          &New {
            author_id: None,
            body: format!("Note {i}"),
          },
        )
        .await
        .unwrap();
        ids.push(note.id().to_string());
      }

      let other_artifact = make_artifact(&conn, &pid).await;
      create(
        &conn,
        EntityType::Artifact,
        &other_artifact,
        &New {
          author_id: None,
          body: "Other".into(),
        },
      )
      .await
      .unwrap();

      let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
      let expected = min_unique_prefix(&refs);
      let got = shortest_prefix(&conn, &artifact_id).await.unwrap();

      assert_eq!(got, expected);
    }

    #[tokio::test]
    async fn it_returns_one_for_artifact_with_no_notes() {
      let (_store, conn, _tmp, pid, _task_id) = setup().await;
      let artifact_id = make_artifact(&conn, &pid).await;

      assert_eq!(shortest_prefix(&conn, &artifact_id).await.unwrap(), 1);
    }
  }

  mod update_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_updates_the_body() {
      let (_store, conn, _tmp, _pid, task_id) = setup().await;
      let note = create(
        &conn,
        EntityType::Task,
        &task_id,
        &New {
          author_id: None,
          body: "Old".into(),
        },
      )
      .await
      .unwrap();

      let updated = update(
        &conn,
        note.id(),
        &Patch {
          body: Some("New".into()),
        },
      )
      .await
      .unwrap();

      assert_eq!(updated.body(), "New");
    }
  }
}
