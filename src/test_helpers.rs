//! Factory helpers for building model instances in tests.

use std::path::PathBuf;

use chrono::Utc;

use crate::{
  config::Settings,
  model::{Artifact, Iteration, Task, iteration, task::Status},
};

/// Create a minimal [`Artifact`] with sensible defaults for the given encoded ID.
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

/// Build a [`Settings`] with resolved storage paths pointing at the given directory.
pub fn make_test_config(project_dir: PathBuf) -> Settings {
  let mut settings = Settings::default();
  settings.storage_mut().resolve_at(project_dir.clone());
  settings.storage_mut().resolve_state_at(project_dir.join("state"));
  settings
}

pub fn make_test_context(base: &std::path::Path) -> crate::cli::AppContext {
  let settings = make_test_config(base.to_path_buf());
  let theme = crate::ui::theme::Theme::default();
  crate::cli::AppContext {
    settings,
    theme,
  }
}

/// Create a minimal [`Iteration`] with sensible defaults for the given encoded ID.
pub fn make_test_iteration(id: &str) -> Iteration {
  let now = Utc::now();
  Iteration {
    completed_at: None,
    created_at: now,
    description: String::new(),
    events: vec![],
    id: id.parse().unwrap(),
    links: vec![],
    metadata: toml::Table::new(),
    phase_count: None,
    status: iteration::Status::Active,
    tags: vec![],
    tasks: vec![],
    title: format!("Iteration {id}"),
    updated_at: now,
  }
}

/// Create a minimal [`Task`] with sensible defaults for the given encoded ID.
pub fn make_test_task(id: &str) -> Task {
  let now = Utc::now();
  Task {
    assigned_to: None,
    created_at: now,
    description: String::new(),
    events: vec![],
    id: id.parse().unwrap(),
    links: vec![],
    metadata: toml::Table::new(),
    notes: vec![],
    phase: None,
    priority: None,
    resolved_at: None,
    status: Status::Open,
    tags: vec![],
    title: format!("Task {id}"),
    updated_at: now,
  }
}
