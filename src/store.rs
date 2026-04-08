//! SQLite-backed store: connection management, migrations, models, and repos.
//!
//! The store opens a per-user libsql database (local file or remote URL),
//! runs schema migrations at startup, and exposes [`repo`] helpers that
//! translate between domain [`model`]s and database rows. When a project has
//! been resolved, [`Db::configure_sync`] wires in transparent import/export
//! against the project's `.gest/` directory.

/// JSON metadata helpers shared by all `meta` subcommands.
pub mod meta;
/// Sequential schema migrations applied at startup.
pub mod migration;
/// Domain model types persisted by the store.
pub mod model;
/// Repository helpers that read and write domain models.
pub mod repo;
/// Parsed representation of search query strings.
pub mod search_query;
/// Import and export between the database and a project's `.gest/` directory.
pub mod sync;

use std::{
  fmt::{self, Debug, Formatter},
  io::Error as IoError,
  path::PathBuf,
  sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
  },
};

use libsql::{Connection, Database, Error as DbError};

use crate::store::model::primitives::Id;

/// Thin wrapper around a [`libsql::Database`] with optional transparent sync.
pub struct Db {
  /// Whether the initial sync import has already run this process.
  imported: AtomicBool,
  inner: Database,
  /// Sync context set after project resolution: `(project_id, gest_dir)`.
  sync_ctx: OnceLock<(Id, PathBuf)>,
}

impl Debug for Db {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("Db").finish_non_exhaustive()
  }
}

impl Db {
  /// Obtain a new connection to the underlying database.
  ///
  /// Each connection has `PRAGMA foreign_keys = ON` enabled so that
  /// `REFERENCES` constraints are enforced.
  pub async fn connect(&self) -> Result<Connection, Error> {
    let conn = self.inner.connect()?;
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;
    Ok(conn)
  }

  /// Configure transparent sync with a `.gest/` directory.
  ///
  /// Must be called after project resolution and before the first
  /// `import_if_needed()` call. Subsequent calls are no-ops.
  pub fn configure_sync(&self, project_id: Id, gest_dir: PathBuf) {
    self.sync_ctx.set((project_id, gest_dir)).ok();
  }

  /// Run the sync import if configured and not yet imported this process.
  ///
  /// Called automatically at application startup; safe to call multiple times
  /// (only the first call actually imports).
  pub async fn import_if_needed(&self) -> Result<(), Error> {
    if let Some((pid, dir)) = self.sync_ctx.get()
      && !self.imported.swap(true, Ordering::SeqCst)
    {
      let conn = self.connect().await?;
      if let Err(e) = sync::import(&conn, pid, dir).await {
        log::warn!("sync import failed: {e}");
      }
    }
    Ok(())
  }

  /// Run the sync export if configured.
  ///
  /// Called at application exit to flush any database changes back to the
  /// `.gest/` directory.
  pub async fn export_if_needed(&self) -> Result<(), Error> {
    if let Some((pid, dir)) = self.sync_ctx.get() {
      let conn = self.connect().await?;
      if let Err(e) = sync::export(&conn, pid, dir).await {
        log::warn!("sync export failed: {e}");
      }
    }
    Ok(())
  }
}

/// Errors that can occur when opening the database store.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// A configuration value (such as the resolved data directory) could not be read.
  #[error(transparent)]
  Config(#[from] crate::config::Error),
  /// The underlying libsql driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// Creating or accessing the on-disk database file failed.
  #[error(transparent)]
  Io(#[from] IoError),
}

/// Open (or create) the database described by `settings`.
///
/// When `database.url` is configured the store connects to that remote database.
/// Otherwise a standalone local SQLite file at `<data_dir>/gest.db` is used.
pub async fn open(settings: &crate::config::Settings) -> Result<Arc<Db>, Error> {
  let db = if let Some(url) = settings.database().url() {
    log::debug!("opening remote database at {url}");
    let auth_token = settings.database().auth_token().clone().unwrap_or_default();
    libsql::Builder::new_remote(url, auth_token).build().await?
  } else {
    let data_dir = settings.storage().data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let path = data_dir.join("gest.db");
    log::debug!("opening local database at {}", path.display());
    libsql::Builder::new_local(path).build().await?
  };

  let store = Arc::new(Db {
    inner: db,
    imported: AtomicBool::new(false),
    sync_ctx: OnceLock::new(),
  });

  let conn = store.connect().await?;
  migration::run(&conn).await?;

  Ok(store)
}

/// Open a temporary local database. Useful for tests that need an `AppContext`
/// but don't exercise persistence.
///
/// Uses a temp file rather than `:memory:` because libsql in-memory databases
/// do not share state across connections.
#[cfg(test)]
pub async fn open_temp() -> Result<(Arc<Db>, tempfile::TempDir), Error> {
  let tmp = tempfile::tempdir()?;
  let path = tmp.path().join("gest-test.db");
  let db = libsql::Builder::new_local(path).build().await?;

  let store = Arc::new(Db {
    inner: db,
    imported: AtomicBool::new(false),
    sync_ctx: OnceLock::new(),
  });

  let conn = store.connect().await?;
  migration::run(&conn).await?;

  Ok((store, tmp))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod open {
    use std::path::PathBuf;

    use super::*;
    use crate::config::Settings;

    fn settings_with_data_dir(dir: PathBuf) -> Settings {
      toml::from_str(&format!("[storage]\ndata_dir = {:?}", dir.to_str().unwrap())).unwrap()
    }

    #[tokio::test]
    async fn it_creates_data_dir_if_missing() {
      let tmp = tempfile::tempdir().unwrap();
      let nested = tmp.path().join("nested").join("dir");
      let settings = settings_with_data_dir(nested.clone());

      let _store = open(&settings).await.unwrap();

      assert!(nested.exists());
    }

    #[tokio::test]
    async fn it_creates_local_db_when_no_url() {
      let tmp = tempfile::tempdir().unwrap();
      let settings = settings_with_data_dir(tmp.path().to_path_buf());

      let store = open(&settings).await.unwrap();
      let conn = store.connect().await.unwrap();

      // Verify we can execute a basic query
      conn
        .execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", ())
        .await
        .unwrap();
      conn.execute("INSERT INTO test (id) VALUES (1)", ()).await.unwrap();
      let mut rows = conn.query("SELECT id FROM test", ()).await.unwrap();
      let row = rows.next().await.unwrap().unwrap();
      let id: i64 = row.get(0).unwrap();
      assert_eq!(id, 1);
    }
  }
}
