// Re-exports include items used only in test code across the crate; allow unused_imports
// to avoid false positives for those items in non-test builds.
#![allow(unused_imports)]

mod artifact;
mod fs;
mod iteration;
mod search;
mod task;

pub use artifact::{
  archive_artifact, artifact_path, create_artifact, list_artifacts, read_artifact, resolve_artifact_id,
  update_artifact, write_artifact,
};
pub use fs::ensure_dirs;
pub use iteration::{
  add_task as add_iteration_task, create_iteration, is_iteration_resolved, list_iterations, read_iteration,
  remove_task as remove_iteration_task, resolve_iteration, resolve_iteration_id, update_iteration, write_iteration,
};
pub use search::{SearchResults, search};
pub use task::{
  create_task, is_task_resolved, list_tasks, read_task, resolve_task, resolve_task_id, update_task, write_task,
};
