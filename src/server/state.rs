//! Server-side application state shared across all request handlers.

use std::path::PathBuf;

/// Shared state available to every handler via Axum's state extractor.
#[derive(Clone, Debug)]
pub struct ServerState {
  pub artifact_dir: PathBuf,
  pub iteration_dir: PathBuf,
  pub task_dir: PathBuf,
}
