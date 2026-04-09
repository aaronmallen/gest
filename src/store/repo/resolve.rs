use libsql::Connection;

use crate::store::{
  Error,
  model::primitives::{EntityType, Id},
};

/// Returns the SQL fragment that filters a table to its "active" set.
///
/// Active means:
/// - artifacts: not archived (`archived_at IS NULL`)
/// - tasks: not in a terminal state (`status NOT IN ('done', 'cancelled')`)
/// - iterations: not in a terminal state (`status NOT IN ('completed', 'cancelled')`)
///
/// Returns `None` for tables without an active concept.
fn active_filter(table: &str) -> Option<&'static str> {
  match table {
    "artifacts" => Some("archived_at IS NULL"),
    "tasks" => Some("status NOT IN ('done', 'cancelled')"),
    "iterations" => Some("status NOT IN ('completed', 'cancelled')"),
    _ => None,
  }
}

/// Query the given table for IDs matching the prefix, optionally restricted
/// to the active set.
async fn query_matches(conn: &Connection, table: &str, prefix: &str, active_only: bool) -> Result<Vec<String>, Error> {
  let sql = if active_only && let Some(filter) = active_filter(table) {
    format!("SELECT id FROM {table} WHERE id LIKE ?1 || '%' AND {filter}")
  } else {
    format!("SELECT id FROM {table} WHERE id LIKE ?1 || '%'")
  };

  let mut rows = conn.query(&sql, [prefix.to_string()]).await?;
  let mut out = Vec::new();
  while let Some(row) = rows.next().await? {
    let id_str: String = row.get(0)?;
    out.push(id_str);
  }
  Ok(out)
}

/// Resolve an ID prefix to a full ID by querying the given table.
///
/// Resolution is two-phase:
/// 1. First, search the active set (non-archived / non-terminal rows). If
///    exactly one match → return it. If more than one → ambiguous error.
/// 2. If zero active matches, fall back to searching all rows. Apply the
///    same one/many/none logic.
///
/// The fallback is silent: a prefix matching one active and one
/// archived/terminal entity returns the active one with no ambiguity error
/// or hint.
pub async fn resolve_id(conn: &Connection, table: &str, prefix: &str) -> Result<Id, Error> {
  log::debug!("repo::resolve::resolve_id");
  Id::validate_prefix(prefix).map_err(Error::InvalidPrefix)?;

  // Phase 1: active set
  let active_matches = query_matches(conn, table, prefix, true).await?;
  match active_matches.len() {
    1 => {
      return active_matches
        .into_iter()
        .next()
        .unwrap()
        .parse::<Id>()
        .map_err(Error::InvalidPrefix);
    }
    n if n > 1 => return Err(Error::Ambiguous(prefix.to_string(), n)),
    _ => {}
  }

  // Phase 2: fallback to all rows
  let all_matches = query_matches(conn, table, prefix, false).await?;
  match all_matches.len() {
    0 => Err(Error::NotFound(format!("id prefix '{prefix}'"))),
    1 => all_matches
      .into_iter()
      .next()
      .unwrap()
      .parse::<Id>()
      .map_err(Error::InvalidPrefix),
    n => Err(Error::Ambiguous(prefix.to_string(), n)),
  }
}

/// Tables to search when resolving an entity across all types.
const ENTITY_TABLES: &[(EntityType, &str)] = &[
  (EntityType::Artifact, "artifacts"),
  (EntityType::Iteration, "iterations"),
  (EntityType::Task, "tasks"),
];

/// Collect matches across all entity tables, optionally restricted to active
/// rows.
async fn collect_entity_matches(
  conn: &Connection,
  prefix: &str,
  active_only: bool,
) -> Result<Vec<(EntityType, Id)>, Error> {
  let mut matches: Vec<(EntityType, Id)> = Vec::new();
  for &(entity_type, table) in ENTITY_TABLES {
    let ids = query_matches(conn, table, prefix, active_only).await?;
    for id_str in ids {
      let id = id_str.parse::<Id>().map_err(Error::InvalidPrefix)?;
      matches.push((entity_type, id));
    }
  }
  Ok(matches)
}

/// Resolve an ID prefix across all entity tables.
///
/// Resolution is two-phase:
/// 1. First, search the active set across artifacts, iterations, and tasks.
///    If exactly one match → return it. If more than one → ambiguous error.
/// 2. If zero active matches, fall back to searching all rows. Apply the
///    same one/many/none logic.
///
/// The fallback is silent: a prefix matching one active and one
/// archived/terminal entity returns the active one with no ambiguity error
/// or hint.
pub async fn resolve_entity(conn: &Connection, prefix: &str) -> Result<(EntityType, Id), Error> {
  log::debug!("repo::resolve::resolve_entity");
  Id::validate_prefix(prefix).map_err(Error::InvalidPrefix)?;

  // Phase 1: active set
  let active_matches = collect_entity_matches(conn, prefix, true).await?;
  match active_matches.len() {
    1 => return Ok(active_matches.into_iter().next().unwrap()),
    n if n > 1 => return Err(Error::Ambiguous(prefix.to_string(), n)),
    _ => {}
  }

  // Phase 2: fallback to all rows
  let all_matches = collect_entity_matches(conn, prefix, false).await?;
  match all_matches.len() {
    0 => Err(Error::NotFound(format!("id prefix '{prefix}'"))),
    1 => Ok(all_matches.into_iter().next().unwrap()),
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

  /// Insert a task with the given id and status. Returns the id.
  async fn insert_task(conn: &Connection, pid: &Id, id: &Id, status: &str) {
    conn
      .execute(
        "INSERT INTO tasks (id, project_id, title, status) VALUES (?1, ?2, ?3, ?4)",
        [id.to_string(), pid.to_string(), "Task".to_string(), status.to_string()],
      )
      .await
      .unwrap();
  }

  /// Insert an artifact with the given id and archived state.
  async fn insert_artifact(conn: &Connection, pid: &Id, id: &Id, archived: bool) {
    if archived {
      conn
        .execute(
          "INSERT INTO artifacts (id, project_id, title, body, archived_at) VALUES (?1, ?2, ?3, ?4, ?5)",
          [
            id.to_string(),
            pid.to_string(),
            "Artifact".to_string(),
            "body".to_string(),
            "2025-01-01T00:00:00Z".to_string(),
          ],
        )
        .await
        .unwrap();
    } else {
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
    }
  }

  /// Build an Id from a fixed 32-character string in the `[k-z]` alphabet.
  fn id(s: &str) -> Id {
    s.parse().unwrap()
  }

  mod resolve_entity_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_falls_back_across_types_to_archived_match() {
      let (_store, conn, _tmp, pid) = setup().await;

      let archived = Id::new();
      insert_artifact(&conn, &pid, &archived, true).await;

      let (entity_type, resolved) = resolve_entity(&conn, &archived.short()).await.unwrap();
      assert_eq!(entity_type, EntityType::Artifact);
      assert_eq!(resolved, archived);
    }

    #[tokio::test]
    async fn it_prefers_active_over_terminal_silently_across_types() {
      let (_store, conn, _tmp, pid) = setup().await;

      let prefix = "klmn";
      let active_task = id("klmnopqrstuvwxyzkkkkkkkkkkkkkkkk");
      let archived_artifact = id("klmnopqrstuvwxyzllllllllllllllll");
      insert_task(&conn, &pid, &active_task, "open").await;
      insert_artifact(&conn, &pid, &archived_artifact, true).await;

      let (entity_type, resolved) = resolve_entity(&conn, prefix).await.unwrap();
      assert_eq!(entity_type, EntityType::Task);
      assert_eq!(resolved, active_task);
    }

    #[tokio::test]
    async fn it_resolves_a_task() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      insert_task(&conn, &pid, &id, "open").await;

      let (entity_type, resolved) = resolve_entity(&conn, &id.short()).await.unwrap();

      assert_eq!(entity_type, EntityType::Task);
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_resolves_an_artifact() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      insert_artifact(&conn, &pid, &id, false).await;

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
    async fn it_returns_error_when_invalid_prefix() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_entity(&conn, "invalid!").await;

      assert!(matches!(result, Err(Error::InvalidPrefix(_))));
    }

    #[tokio::test]
    async fn it_returns_error_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_entity(&conn, "kkkkkkkk").await;

      assert!(matches!(result, Err(Error::NotFound(_))));
    }
  }

  mod resolve_id_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn it_falls_back_to_archived_artifact_when_no_active_match() {
      let (_store, conn, _tmp, pid) = setup().await;

      let archived = Id::new();
      insert_artifact(&conn, &pid, &archived, true).await;

      let resolved = resolve_id(&conn, "artifacts", &archived.short()).await.unwrap();
      assert_eq!(resolved, archived);
    }

    #[tokio::test]
    async fn it_falls_back_to_terminal_when_no_active_match() {
      let (_store, conn, _tmp, pid) = setup().await;

      let done = Id::new();
      insert_task(&conn, &pid, &done, "done").await;

      let resolved = resolve_id(&conn, "tasks", &done.short()).await.unwrap();
      assert_eq!(resolved, done);
    }

    #[tokio::test]
    async fn it_prefers_active_over_archived_silently_on_collision() {
      let (_store, conn, _tmp, pid) = setup().await;

      // Two artifacts with a shared 4-char prefix: one active, one archived.
      let prefix = "klmn";
      let active = id("klmnopqrstuvwxyzkkkkkkkkkkkkkkkk");
      let archived = id("klmnopqrstuvwxyzllllllllllllllll");
      insert_artifact(&conn, &pid, &active, false).await;
      insert_artifact(&conn, &pid, &archived, true).await;

      let resolved = resolve_id(&conn, "artifacts", prefix).await.unwrap();
      assert_eq!(resolved, active);
    }

    #[tokio::test]
    async fn it_resolves_a_full_id() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      insert_task(&conn, &pid, &id, "open").await;

      let resolved = resolve_id(&conn, "tasks", &id.to_string()).await.unwrap();
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_resolves_a_prefix() {
      let (_store, conn, _tmp, pid) = setup().await;

      let id = Id::new();
      insert_task(&conn, &pid, &id, "open").await;

      let resolved = resolve_id(&conn, "tasks", &id.short()).await.unwrap();
      assert_eq!(resolved, id);
    }

    #[tokio::test]
    async fn it_returns_ambiguous_within_active_set() {
      let (_store, conn, _tmp, pid) = setup().await;

      let prefix = "klmn";
      let a = id("klmnopqrstuvwxyzkkkkkkkkkkkkkkkk");
      let b = id("klmnopqrstuvwxyzllllllllllllllll");
      insert_task(&conn, &pid, &a, "open").await;
      insert_task(&conn, &pid, &b, "open").await;

      let result = resolve_id(&conn, "tasks", prefix).await;
      assert!(matches!(result, Err(Error::Ambiguous(_, 2))));
    }

    #[tokio::test]
    async fn it_returns_ambiguous_within_terminal_when_zero_active() {
      let (_store, conn, _tmp, pid) = setup().await;

      let prefix = "klmn";
      let a = id("klmnopqrstuvwxyzkkkkkkkkkkkkkkkk");
      let b = id("klmnopqrstuvwxyzllllllllllllllll");
      insert_task(&conn, &pid, &a, "done").await;
      insert_task(&conn, &pid, &b, "cancelled").await;

      let result = resolve_id(&conn, "tasks", prefix).await;
      assert!(matches!(result, Err(Error::Ambiguous(_, 2))));
    }

    #[tokio::test]
    async fn it_returns_error_when_invalid_prefix() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_id(&conn, "tasks", "invalid!").await;
      assert!(matches!(result, Err(Error::InvalidPrefix(_))));
    }

    #[tokio::test]
    async fn it_returns_error_when_not_found() {
      let (_store, conn, _tmp, _pid) = setup().await;

      let result = resolve_id(&conn, "tasks", "kkkkkkkk").await;
      assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn it_returns_unique_active_match() {
      let (_store, conn, _tmp, pid) = setup().await;

      let active = Id::new();
      insert_task(&conn, &pid, &active, "open").await;

      let resolved = resolve_id(&conn, "tasks", &active.short()).await.unwrap();
      assert_eq!(resolved, active);
    }
  }
}
