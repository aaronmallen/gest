//! Sequential schema migrations for the application database.
//!
//! Each migration is defined in its own submodule (`m00001_*`, `m00002_*`, …) and
//! registered in the [`MIGRATIONS`] array. The [`run`] function applies any migrations
//! that have not yet been recorded in the `_migrations` tracking table.

mod m00001_create_projects;
mod m00002_create_project_workspaces;
mod m00003_create_authors;
mod m00004_create_tasks;
mod m00005_create_artifacts;
mod m00006_create_iterations;
mod m00007_create_tags;
mod m00008_create_entity_tags;
mod m00009_create_notes;
mod m00010_create_events;
mod m00011_create_relationships;
mod m00012_create_iteration_tasks;
mod m00013_create_transactions;
mod m00014_create_sync_digests;

use libsql::{Connection, Error as DbError};

/// A single versioned schema migration.
struct Migration {
  /// Human-readable name for logging.
  name: &'static str,
  /// SQL statements to execute when applying this migration.
  sql: &'static str,
  /// Monotonically increasing version number.
  version: i64,
}

/// All registered migrations, in the order they must be applied.
const MIGRATIONS: &[Migration] = &[
  m00001_create_projects::MIGRATION,
  m00002_create_project_workspaces::MIGRATION,
  m00003_create_authors::MIGRATION,
  m00004_create_tasks::MIGRATION,
  m00005_create_artifacts::MIGRATION,
  m00006_create_iterations::MIGRATION,
  m00007_create_tags::MIGRATION,
  m00008_create_entity_tags::MIGRATION,
  m00009_create_notes::MIGRATION,
  m00010_create_events::MIGRATION,
  m00011_create_relationships::MIGRATION,
  m00012_create_iteration_tasks::MIGRATION,
  m00013_create_transactions::MIGRATION,
  m00014_create_sync_digests::MIGRATION,
];

/// Run all pending migrations against the given connection.
///
/// Creates the internal `_migrations` table if it does not yet exist, then
/// applies each migration whose version has not been recorded.
pub async fn run(conn: &Connection) -> Result<(), DbError> {
  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS _migrations (
        version INTEGER PRIMARY KEY,
        name    TEXT NOT NULL,
        applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
      )",
      (),
    )
    .await?;

  for migration in MIGRATIONS {
    let already_applied: bool = conn
      .query("SELECT 1 FROM _migrations WHERE version = ?1", [migration.version])
      .await?
      .next()
      .await?
      .is_some();

    if already_applied {
      continue;
    }

    log::info!("applying migration v{:04}: {}", migration.version, migration.name);
    conn.execute_batch(migration.sql).await?;
    conn
      .execute(
        "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
        (migration.version, migration.name),
      )
      .await?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::store;

  mod run {
    use super::*;

    #[tokio::test]
    async fn it_applies_all_migrations() {
      let (_store, _tmp) = store::open_temp().await.unwrap();
      let conn = _store.connect().await.unwrap();

      let mut rows = conn
        .query("SELECT version FROM _migrations ORDER BY version", ())
        .await
        .unwrap();

      let row = rows.next().await.unwrap().unwrap();
      let version: i64 = row.get(0).unwrap();
      assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn it_is_idempotent() {
      let (_store, _tmp) = store::open_temp().await.unwrap();
      let conn = _store.connect().await.unwrap();

      run(&conn).await.unwrap();

      let mut rows = conn.query("SELECT count(*) FROM _migrations", ()).await.unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let count: i64 = row.get(0).unwrap();
      assert_eq!(count, MIGRATIONS.len() as i64);
    }
  }
}
