use std::{collections::HashSet, fs};

use chrono::Utc;

use super::{
  fs::{ensure_dirs, move_entity_file, resolve_id},
  helpers::{load_entities_from_dirs, read_entity_file},
};
use crate::{
  config::Settings,
  model::{Id, Iteration, IterationFilter, IterationPatch, NewIteration, Task},
};

/// Compute how many distinct phases the iteration's tasks span by reading each task file.
fn compute_phase_count(config: &Settings, tasks: &[String]) -> usize {
  let mut phases = HashSet::new();
  for task_ref in tasks {
    let task_id_str = task_ref.strip_prefix("tasks/").unwrap_or(task_ref);
    if let Ok(id) = task_id_str.parse()
      && let Ok(task) = super::read_task(config, &id)
    {
      phases.insert(task.phase.unwrap_or(0));
    }
  }
  phases.len()
}

/// Append a task reference to an iteration (idempotent).
pub fn add_task(config: &Settings, iteration_id: &Id, task_id: &str) -> super::Result<Iteration> {
  let mut iteration = read_iteration(config, iteration_id)?;
  if !iteration.tasks.contains(&task_id.to_string()) {
    iteration.tasks.push(task_id.to_string());
    iteration.phase_count = Some(compute_phase_count(config, &iteration.tasks));
    iteration.updated_at = Utc::now();
    write_iteration(config, &iteration)?;
  }
  Ok(iteration)
}

/// Persist a new iteration, resolving it immediately if the status is terminal.
pub fn create_iteration(config: &Settings, new: NewIteration) -> super::Result<Iteration> {
  let now = Utc::now();
  let iteration = Iteration {
    completed_at: None,
    created_at: now,
    description: new.description,
    id: Id::new(),
    links: new.links,
    metadata: new.metadata,
    phase_count: Some(0),
    status: new.status,
    tags: new.tags,
    tasks: new.tasks,
    title: new.title,
    updated_at: now,
  };

  write_iteration(config, &iteration)?;

  if iteration.status.is_terminal() {
    resolve_iteration(config, &iteration.id)?;
    return read_iteration(config, &iteration.id);
  }

  Ok(iteration)
}

/// Check whether an iteration has been moved to the resolved directory.
pub fn is_iteration_resolved(config: &Settings, id: &Id) -> bool {
  let resolved_path = config.iteration_dir().join(format!("resolved/{id}.toml"));
  let active_path = config.iteration_dir().join(format!("{id}.toml"));
  resolved_path.exists() && !active_path.exists()
}

/// List iterations matching the given filter criteria.
pub fn list_iterations(config: &Settings, filter: &IterationFilter) -> super::Result<Vec<Iteration>> {
  let parse = |content: &str| Ok(toml::from_str::<Iteration>(content)?);
  let mut iterations = load_entities_from_dirs(
    config.iteration_dir(),
    &config.iteration_dir().join("resolved"),
    "toml",
    false,
    filter.all,
    parse,
  )?;

  iterations.retain(|iteration| {
    if let Some(ref status) = filter.status
      && &iteration.status != status
    {
      return false;
    }
    if let Some(ref tag) = filter.tag
      && !iteration.tags.contains(tag)
    {
      return false;
    }
    true
  });

  Ok(iterations)
}

/// Load a single iteration by exact ID, checking both active and resolved directories.
pub fn read_iteration(config: &Settings, id: &Id) -> super::Result<Iteration> {
  let active = config.iteration_dir().join(format!("{id}.toml"));
  let resolved = config.iteration_dir().join(format!("resolved/{id}.toml"));

  read_entity_file(&active, &resolved, "resolved", "Iteration", id, |content| {
    Ok(toml::from_str::<Iteration>(content)?)
  })
}

/// Load all tasks referenced by an iteration, silently skipping any that
/// cannot be parsed or read.
pub fn read_iteration_tasks(config: &Settings, iteration: &Iteration) -> Vec<Task> {
  let mut tasks = Vec::new();
  for task_ref in &iteration.tasks {
    let task_id_str = task_ref.strip_prefix("tasks/").unwrap_or(task_ref);
    if let Ok(task_id) = task_id_str.parse()
      && let Ok(task) = super::read_task(config, &task_id)
    {
      tasks.push(task);
    }
  }
  tasks
}

/// Remove a task reference from an iteration.
pub fn remove_task(config: &Settings, iteration_id: &Id, task_id: &str) -> super::Result<Iteration> {
  let mut iteration = read_iteration(config, iteration_id)?;
  iteration.tasks.retain(|t| t != task_id);
  iteration.phase_count = Some(compute_phase_count(config, &iteration.tasks));
  iteration.updated_at = Utc::now();
  write_iteration(config, &iteration)?;
  Ok(iteration)
}

/// Move an iteration to the resolved directory, setting its `completed_at` timestamp.
pub fn resolve_iteration(config: &Settings, id: &Id) -> super::Result<()> {
  let mut iteration = read_iteration(config, id)?;
  let now = Utc::now();
  iteration.completed_at = Some(now);
  iteration.updated_at = now;

  let content = toml::to_string(&iteration)?;
  move_entity_file(
    config,
    &content,
    &config.iteration_dir().join(format!("resolved/{id}.toml")),
    &config.iteration_dir().join(format!("{id}.toml")),
  )?;

  Ok(())
}

/// Resolve a short ID prefix to a full iteration [`Id`].
pub fn resolve_iteration_id(config: &Settings, prefix: &str, include_resolved: bool) -> super::Result<Id> {
  log::debug!("resolving iteration ID prefix '{prefix}'");
  resolve_id(
    config.iteration_dir(),
    Some(&config.iteration_dir().join("resolved")),
    "toml",
    prefix,
    include_resolved,
    "Iteration",
  )
}

/// Apply a partial update to an existing iteration, moving it between active/resolved as needed.
pub fn update_iteration(config: &Settings, id: &Id, patch: IterationPatch) -> super::Result<Iteration> {
  let mut iteration = read_iteration(config, id)?;
  let was_resolved = is_iteration_resolved(config, id);

  if let Some(description) = patch.description {
    iteration.description = description;
  }
  if let Some(metadata) = patch.metadata {
    iteration.metadata = metadata;
  }
  if let Some(status) = patch.status {
    iteration.status = status;
  }
  if let Some(tags) = patch.tags {
    iteration.tags = tags;
  }
  if let Some(title) = patch.title {
    iteration.title = title;
  }

  iteration.updated_at = Utc::now();

  if iteration.status.is_terminal() && !was_resolved {
    iteration.completed_at = Some(iteration.updated_at);
    let content = toml::to_string(&iteration)?;
    move_entity_file(
      config,
      &content,
      &config.iteration_dir().join(format!("resolved/{id}.toml")),
      &config.iteration_dir().join(format!("{id}.toml")),
    )?;
  } else if !iteration.status.is_terminal() && was_resolved {
    iteration.completed_at = None;
    let content = toml::to_string(&iteration)?;
    move_entity_file(
      config,
      &content,
      &config.iteration_dir().join(format!("{id}.toml")),
      &config.iteration_dir().join(format!("resolved/{id}.toml")),
    )?;
  } else {
    write_iteration(config, &iteration)?;
  }

  Ok(iteration)
}

/// Serialize and write an iteration to the active iterations directory.
pub fn write_iteration(config: &Settings, iteration: &Iteration) -> super::Result<()> {
  ensure_dirs(config)?;
  let path = config.iteration_dir().join(format!("{}.toml", iteration.id));
  let content = toml::to_string(iteration)?;
  log::trace!("writing iteration {} to {}", iteration.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{
    config::Settings,
    model::{Iteration, IterationFilter, iteration::Status},
  };

  fn make_config(base: &std::path::Path) -> Settings {
    crate::test_helpers::make_test_config(base.to_path_buf())
  }

  fn make_test_iteration(id: &str, title: &str) -> Iteration {
    Iteration {
      title: title.to_string(),
      ..crate::test_helpers::make_test_iteration(id)
    }
  }

  mod add_task {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_a_task_reference() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      let updated = crate::store::add_iteration_task(
        &make_config(dir.path()),
        &iteration.id,
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
      )
      .unwrap();
      assert_eq!(updated.tasks, vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]);
    }

    #[test]
    fn it_is_idempotent() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      crate::store::add_iteration_task(
        &make_config(dir.path()),
        &iteration.id,
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
      )
      .unwrap();
      let updated = crate::store::add_iteration_task(
        &make_config(dir.path()),
        &iteration.id,
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
      )
      .unwrap();
      assert_eq!(updated.tasks.len(), 1);
    }
  }

  mod create_iteration {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::NewIteration;

    #[test]
    fn it_creates_an_iteration() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(&make_config(dir.path())).unwrap();

      let new = NewIteration {
        title: "Sprint 1".to_string(),
        ..Default::default()
      };

      let iteration = crate::store::create_iteration(&make_config(dir.path()), new).unwrap();
      assert_eq!(iteration.title, "Sprint 1");
      assert_eq!(iteration.status, Status::Active);
      assert!(iteration.completed_at.is_none());
    }
  }

  mod list_iterations {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_excludes_resolved_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_config(dir.path()), &iteration.id).unwrap();

      let filter = IterationFilter::default();
      let iterations = crate::store::list_iterations(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(iterations.len(), 0);
    }

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      let mut i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Failed");
      i2.status = Status::Failed;
      crate::store::write_iteration(&make_config(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_config(dir.path()), &i2).unwrap();

      let filter = IterationFilter {
        status: Some(Status::Failed),
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Failed");
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged");
      i1.tags = vec!["sprint".to_string()];
      let i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Untagged");
      crate::store::write_iteration(&make_config(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_config(dir.path()), &i2).unwrap();

      let filter = IterationFilter {
        tag: Some("sprint".to_string()),
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Tagged");
    }

    #[test]
    fn it_includes_resolved_when_all() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_config(dir.path()), &iteration.id).unwrap();

      let filter = IterationFilter {
        all: true,
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(iterations.len(), 1);
    }

    #[test]
    fn it_returns_active_iterations() {
      let dir = tempfile::tempdir().unwrap();
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "One");
      let i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Two");
      crate::store::write_iteration(&make_config(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_config(dir.path()), &i2).unwrap();

      let filter = IterationFilter::default();
      let iterations = crate::store::list_iterations(&make_config(dir.path()), &filter).unwrap();

      assert_eq!(iterations.len(), 2);
    }
  }

  mod read_iteration {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_reads_resolved() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_config(dir.path()), &iteration.id).unwrap();

      let loaded = crate::store::read_iteration(&make_config(dir.path()), &iteration.id).unwrap();
      assert_eq!(loaded.title, "Test");
      assert!(loaded.completed_at.is_some());
    }

    #[test]
    fn it_roundtrips() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      let loaded = crate::store::read_iteration(&make_config(dir.path()), &iteration.id).unwrap();
      assert_eq!(iteration, loaded);
    }
  }

  mod remove_task {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_a_task_reference() {
      let dir = tempfile::tempdir().unwrap();
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      iteration.tasks = vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string()];
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      let updated = crate::store::remove_iteration_task(
        &make_config(dir.path()),
        &iteration.id,
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
      )
      .unwrap();
      assert_eq!(updated.tasks.len(), 0);
    }
  }

  mod update_iteration {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::IterationPatch;

    #[test]
    fn it_resolves_on_terminal_status() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      let patch = IterationPatch {
        status: Some(Status::Completed),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_config(dir.path()), &iteration.id, patch).unwrap();

      assert!(updated.completed_at.is_some());
      assert!(crate::store::is_iteration_resolved(
        &make_config(dir.path()),
        &iteration.id
      ));
    }

    #[test]
    fn it_unresolves_on_active_status() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_config(dir.path()), &iteration.id).unwrap();

      let patch = IterationPatch {
        status: Some(Status::Active),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_config(dir.path()), &iteration.id, patch).unwrap();

      assert!(updated.completed_at.is_none());
      assert!(!crate::store::is_iteration_resolved(
        &make_config(dir.path()),
        &iteration.id
      ));
    }

    #[test]
    fn it_updates_title() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Old Title");
      crate::store::write_iteration(&make_config(dir.path()), &iteration).unwrap();

      let patch = IterationPatch {
        title: Some("New Title".to_string()),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_config(dir.path()), &iteration.id, patch).unwrap();

      assert_eq!(updated.title, "New Title");
    }
  }
}
