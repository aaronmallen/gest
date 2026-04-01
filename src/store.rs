//! Persistence layer for reading, writing, and querying entities on disk.

#![allow(unused_imports)]

mod artifact;
pub mod artifact_meta;
mod fs;
mod helpers;
mod iteration;
pub mod meta;
mod orchestration;
mod search;
mod task;

use std::io;

/// Errors that can occur during store operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  Generic(String),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  TomlDe(#[from] toml::de::Error),
  #[error(transparent)]
  TomlSer(#[from] toml::ser::Error),
  #[error(transparent)]
  Yaml(#[from] yaml_serde::Error),
}

impl Error {
  /// Construct a free-form error from any string-like message.
  pub fn generic(msg: impl Into<String>) -> Self {
    Self::Generic(msg.into())
  }
}

/// Convenience alias for store operations.
pub type Result<T> = std::result::Result<T, Error>;

pub use artifact::{
  archive_artifact, create_artifact, list_artifacts, read_artifact, resolve_artifact_id, update_artifact,
  write_artifact,
};
pub use fs::ensure_dirs;
pub use iteration::{
  add_task as add_iteration_task, create_iteration, is_iteration_resolved, list_iterations, read_iteration,
  read_iteration_tasks, remove_task as remove_iteration_task, resolve_iteration, resolve_iteration_id,
  update_iteration, write_iteration,
};
pub use orchestration::{
  AdvanceSummary, IterationProgress, OverallProgress, PhaseProgress, advance_phase, claim_task, iteration_status,
  next_available_task,
};
pub use search::{SearchResults, search};
pub use task::{
  ResolvedBlocking, create_task, is_task_resolved, list_tasks, read_task, resolve_blocking, resolve_blocking_batch,
  resolve_task, resolve_task_id, update_task, write_task,
};
