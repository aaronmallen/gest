use std::path::Path;

use chrono::Utc;

use crate::{
  config::{Config, StorageConfig},
  model::{Artifact, Task, task::Status},
};

/// Create a test `Config` whose `data_dir` points at the given directory.
/// Also calls `ensure_dirs` so the store subdirectories exist.
pub fn make_test_config(dir: &Path) -> Config {
  crate::store::ensure_dirs(dir).unwrap();
  Config {
    storage: StorageConfig {
      data_dir: Some(dir.to_path_buf()),
    },
    ..Config::default()
  }
}

/// Create a minimal `Artifact` with sensible defaults. `id` must be a valid
/// 32-character lowercase hex string.
pub fn make_test_artifact(id: &str) -> Artifact {
  let now = Utc::now();
  Artifact {
    archived_at: None,
    body: String::new(),
    created_at: now,
    id: id.parse().unwrap(),
    kind: None,
    metadata: yaml_serde::Mapping::new(),
    tags: vec![],
    title: format!("Artifact {id}"),
    updated_at: now,
  }
}

/// Create a minimal `Task` with sensible defaults. `id` must be a valid
/// 32-character lowercase hex string.
pub fn make_test_task(id: &str) -> Task {
  let now = Utc::now();
  Task {
    created_at: now,
    description: String::new(),
    id: id.parse().unwrap(),
    links: vec![],
    metadata: toml::Table::new(),
    resolved_at: None,
    status: Status::Open,
    tags: vec![],
    title: format!("Task {id}"),
    updated_at: now,
  }
}
