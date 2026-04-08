//! Top-level coordinator for the per-entity sync layout (ADR-0016).
//!
//! Walks every entity adapter under [`super::entities`] in a fixed dependency
//! order. Authors are processed first because every other entity that carries
//! an `author_id` foreign-keys back to them; tags come next so per-entity
//! adapters can resolve embedded tag labels; then the project-scoped entities
//! land in an order that satisfies the remaining FKs.

use std::path::Path;

use libsql::Connection;

use super::{Error, entities};
use crate::store::model::primitives::Id;

/// Import every per-entity file under `gest_dir` into SQLite.
pub async fn import_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  entities::author::read_all(conn, project_id, gest_dir).await?;
  entities::project::read_all(conn, project_id, gest_dir).await?;
  entities::tag::read_all(conn, project_id, gest_dir).await?;
  entities::task::read_all(conn, project_id, gest_dir).await?;
  entities::artifact::read_all(conn, project_id, gest_dir).await?;
  entities::iteration::read_all(conn, project_id, gest_dir).await?;
  entities::relationship::read_all(conn, project_id, gest_dir).await?;
  entities::event::read_all(conn, project_id, gest_dir).await?;
  Ok(())
}

/// Export every entity in SQLite to its per-entity file under `gest_dir`.
pub async fn export_all(conn: &Connection, project_id: &Id, gest_dir: &Path) -> Result<(), Error> {
  entities::project::write_all(conn, project_id, gest_dir).await?;
  entities::author::write_all(conn, project_id, gest_dir).await?;
  entities::tag::write_all(conn, project_id, gest_dir).await?;
  entities::task::write_all(conn, project_id, gest_dir).await?;
  entities::artifact::write_all(conn, project_id, gest_dir).await?;
  entities::iteration::write_all(conn, project_id, gest_dir).await?;
  entities::relationship::write_all(conn, project_id, gest_dir).await?;
  entities::event::write_all(conn, project_id, gest_dir).await?;
  Ok(())
}
