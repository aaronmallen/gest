//! Local sync mirror for `.gest/` directories.
//!
//! When a project is in local mode (has a `.gest/` directory), this module
//! handles bidirectional sync between SQLite and the per-entity YAML/markdown
//! files described in ADR-0016. The orchestrator under [`orchestrator`] walks
//! every entity adapter under [`entities`] in dependency order.

pub mod digest;
mod entities;
mod orchestrator;
pub mod paths;
pub mod tombstone;
pub mod yaml;

use std::path::{Path, PathBuf};

use libsql::Connection;

use crate::store::{Error, model::primitives::Id};

/// Sync state from `.gest/` into the database.
pub async fn import(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  orchestrator::import_all(conn, project_id, gest_dir).await
}

/// Sync state from the database out to `.gest/`.
pub async fn export(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  orchestrator::export_all(conn, project_id, gest_dir).await
}

/// Find the `.gest` directory for a project, if it exists.
pub fn find_gest_dir(root: &Path) -> Option<PathBuf> {
  let candidate = root.join(".gest");
  if candidate.is_dir() { Some(candidate) } else { None }
}
