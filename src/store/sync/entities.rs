//! Per-entity sync adapters.
//!
//! Each submodule owns one entity type's read/write logic for the per-entity
//! `.gest/` layout (ADR-0016). Every adapter exposes the same two functions
//! so [`super::orchestrator`] can call them through a uniform shape:
//!
//! - `read_all(conn, project_id, gest_dir)` — import every file of this entity
//!   type from disk into SQLite.
//! - `write_all(conn, project_id, gest_dir)` — export every row of this entity
//!   type from SQLite to disk.

pub mod artifact;
pub mod author;
pub mod event;
pub mod iteration;
pub mod project;
pub mod relationship;
pub mod tag;
pub mod task;
