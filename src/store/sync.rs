//! Local sync mirror for `.gest/` directories.
//!
//! When a project is in local mode (has a `.gest/` directory), this module
//! handles bidirectional sync between SQLite and the per-entity YAML/markdown
//! files described in ADR-0016. The orchestrator under [`orchestrator`] walks
//! every entity adapter under [`entities`] in dependency order.

mod digest;
mod entities;
mod orchestrator;
pub mod paths;
mod tombstone;
pub mod yaml;

use std::{
  io::Error as IoError,
  path::{Path, PathBuf},
};

use libsql::{Connection, Error as DbError};

use crate::store::model::primitives::Id;

/// Errors that can occur during sync operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// The underlying database driver returned an error.
  #[error(transparent)]
  Database(#[from] DbError),
  /// A filesystem I/O error.
  #[error(transparent)]
  Io(#[from] IoError),
  /// A model conversion error.
  #[error(transparent)]
  Model(#[from] crate::store::model::Error),
  /// A JSON serialization error (used by metadata blobs that remain JSON).
  #[error(transparent)]
  Serialization(#[from] serde_json::Error),
  /// A YAML serialization or deserialization error.
  #[error(transparent)]
  Yaml(#[from] yaml_serde::Error),
}

/// Sync state from `.gest/` into the database.
pub async fn import(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  orchestrator::import_all(conn, project_id, gest_dir).await
}

/// Sync state from the database out to `.gest/`.
pub async fn export(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  orchestrator::export_all(conn, project_id, gest_dir).await
}

/// Perform a full bidirectional sync for a project in local mode.
///
/// 1. Import: read files, import any newer than DB
/// 2. Export: write DB state to files
pub async fn sync(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  import(conn, project_id, gest_dir).await?;
  export(conn, project_id, gest_dir).await?;
  Ok(())
}

/// Find the `.gest` directory for a project, if it exists.
pub fn find_gest_dir(root: &Path) -> Option<PathBuf> {
  let candidate = root.join(".gest");
  if candidate.is_dir() { Some(candidate) } else { None }
}
