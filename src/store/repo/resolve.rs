use libsql::{Connection, Error as DbError};

use crate::store::model::primitives::{EntityType, Id};

/// Errors that can occur during ID prefix resolution.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Multiple entities matched the given prefix.
  #[error("ambiguous id prefix '{0}': matches {1} entities")]
  Ambiguous(String, usize),
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// The given prefix is invalid.
  #[error("{0}")]
  InvalidPrefix(String),
  /// No entity matched the given prefix.
  #[error("no match for id prefix '{0}'")]
  NotFound(String),
}

/// Resolve an ID prefix to a full ID by querying the given table.
///
/// Returns the full ID if exactly one row matches. Returns an error if
/// zero or more than one row matches.
pub async fn resolve_id(conn: &Connection, table: &str, prefix: &str) -> Result<Id, Error> {
  Id::validate_prefix(prefix).map_err(Error::InvalidPrefix)?;

  let sql = format!("SELECT id FROM {table} WHERE id LIKE ?1 || '%'");
  let mut rows = conn.query(&sql, [prefix.to_string()]).await?;

  let first = rows.next().await?;
  let Some(first) = first else {
    return Err(Error::NotFound(prefix.to_string()));
  };

  let id_str: String = first.get(0)?;

  // Check for ambiguity
  if rows.next().await?.is_some() {
    // Count total matches
    let count_sql = format!("SELECT count(*) FROM {table} WHERE id LIKE ?1 || '%'");
    let mut count_rows = conn.query(&count_sql, [prefix.to_string()]).await?;
    let count: i64 = count_rows.next().await?.map(|r| r.get(0)).transpose()?.unwrap_or(0);
    return Err(Error::Ambiguous(prefix.to_string(), count as usize));
  }

  id_str.parse::<Id>().map_err(Error::InvalidPrefix)
}

/// Tables to search when resolving an entity across all types.
const ENTITY_TABLES: &[(EntityType, &str)] = &[
  (EntityType::Artifact, "artifacts"),
  (EntityType::Iteration, "iterations"),
  (EntityType::Task, "tasks"),
];

/// Resolve an ID prefix across all entity tables.
///
/// Searches artifacts, iterations, and tasks. Returns the entity type and
/// full ID if exactly one match is found across all tables. Returns an error
/// if zero or more than one entity matches.
pub async fn resolve_entity(conn: &Connection, prefix: &str) -> Result<(EntityType, Id), Error> {
  Id::validate_prefix(prefix).map_err(Error::InvalidPrefix)?;

  let mut matches: Vec<(EntityType, Id)> = Vec::new();
  for &(entity_type, table) in ENTITY_TABLES {
    let sql = format!("SELECT id FROM {table} WHERE id LIKE ?1 || '%'");
    let mut rows = conn.query(&sql, [prefix.to_string()]).await?;
    while let Some(row) = rows.next().await? {
      let id_str: String = row.get(0)?;
      let id = id_str.parse::<Id>().map_err(Error::InvalidPrefix)?;
      matches.push((entity_type, id));
    }
  }

  match matches.len() {
    0 => Err(Error::NotFound(prefix.to_string())),
    1 => Ok(matches.into_iter().next().unwrap()),
    n => Err(Error::Ambiguous(prefix.to_string(), n)),
  }
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

    let project = Project::new("/tmp/resolve-test".into());
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

  mod resolve_id_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_resolves_a_full_id() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      let resolved = resolve_id(&conn, "tasks", &id.to_string()).await.unwrap();
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_resolves_a_prefix() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      let resolved = resolve_id(&conn, "tasks", &id.short()).await.unwrap();
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_returns_error_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_id(&conn, "tasks", "kkkkkkkk").await;
      assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn it_returns_error_when_invalid_prefix() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_id(&conn, "tasks", "invalid!").await;
      assert!(matches!(result, Err(Error::InvalidPrefix(_))));
    }
  }

  mod resolve_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_resolves_a_task() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      conn
        .execute(
          "INSERT INTO tasks (id, project_id, title) VALUES (?1, ?2, ?3)",
          [id.to_string(), pid.to_string(), "Task".to_string()],
        )
        .await
        .unwrap();

      let (entity_type, resolved) = resolve_entity(&conn, &id.short()).await.unwrap();

      assert_eq!(entity_type, EntityType::Task);
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_resolves_an_artifact() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      conn
        .execute(
          "INSERT INTO artifacts (id, project_id, title, body) VALUES (?1, ?2, ?3, ?4)",
          [
            id.to_string(),
            pid.to_string(),
            "Artifact".to_string(),
            "body".to_string(),
          ],
        )
        .await
        .unwrap();

      let (entity_type, resolved) = resolve_entity(&conn, &id.short()).await.unwrap();

      assert_eq!(entity_type, EntityType::Artifact);
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_resolves_an_iteration() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      conn
        .execute(
          "INSERT INTO iterations (id, project_id, title, status) VALUES (?1, ?2, ?3, ?4)",
          [
            id.to_string(),
            pid.to_string(),
            "Iteration".to_string(),
            "open".to_string(),
          ],
        )
        .await
        .unwrap();

      let (entity_type, resolved) = resolve_entity(&conn, &id.short()).await.unwrap();

      assert_eq!(entity_type, EntityType::Iteration);
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_returns_error_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_entity(&conn, "kkkkkkkk").await;

      assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn it_returns_error_when_invalid_prefix() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_entity(&conn, "invalid!").await;

      assert!(matches!(result, Err(Error::InvalidPrefix(_))));
    }
  }
}
