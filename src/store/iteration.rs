use std::{fs, path::Path};

use chrono::Utc;

use super::{
  Error,
  fs::{ensure_dirs, move_entity_file, read_dir_files, resolve_id},
};
use crate::{
  config::storage::DataLayout,
  model::{Id, Iteration, IterationFilter, IterationPatch, NewIteration, Task},
};

/// Append a task reference to an iteration (idempotent).
pub fn add_task(layout: &DataLayout, iteration_id: &Id, task_id: &str) -> super::Result<Iteration> {
  let mut iteration = read_iteration(layout, iteration_id)?;
  if !iteration.tasks.contains(&task_id.to_string()) {
    iteration.tasks.push(task_id.to_string());
    iteration.updated_at = Utc::now();
    write_iteration(layout, &iteration)?;
  }
  Ok(iteration)
}

/// Persist a new iteration, resolving it immediately if the status is terminal.
pub fn create_iteration(layout: &DataLayout, new: NewIteration) -> super::Result<Iteration> {
  let now = Utc::now();
  let iteration = Iteration {
    completed_at: None,
    created_at: now,
    description: new.description,
    id: Id::new(),
    links: new.links,
    metadata: new.metadata,
    status: new.status,
    tags: new.tags,
    tasks: new.tasks,
    title: new.title,
    updated_at: now,
  };

  write_iteration(layout, &iteration)?;

  if iteration.status.is_terminal() {
    resolve_iteration(layout, &iteration.id)?;
    return read_iteration(layout, &iteration.id);
  }

  Ok(iteration)
}

/// Check whether an iteration has been moved to the resolved directory.
pub fn is_iteration_resolved(layout: &DataLayout, id: &Id) -> bool {
  let resolved_path = layout.iteration_dir().join(format!("resolved/{id}.toml"));
  let active_path = layout.iteration_dir().join(format!("{id}.toml"));
  resolved_path.exists() && !active_path.exists()
}

/// List iterations matching the given filter criteria.
pub fn list_iterations(layout: &DataLayout, filter: &IterationFilter) -> super::Result<Vec<Iteration>> {
  let mut iterations = Vec::new();

  for path in read_dir_files(layout.iteration_dir(), "toml")? {
    let content = fs::read_to_string(&path)?;
    let iteration: Iteration = toml::from_str(&content)?;
    iterations.push(iteration);
  }

  if filter.all {
    for path in read_dir_files(&layout.iteration_dir().join("resolved"), "toml")? {
      let content = fs::read_to_string(&path)?;
      let iteration: Iteration = toml::from_str(&content)?;
      iterations.push(iteration);
    }
  }

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
pub fn read_iteration(layout: &DataLayout, id: &Id) -> super::Result<Iteration> {
  let active = layout.iteration_dir().join(format!("{id}.toml"));
  let resolved = layout.iteration_dir().join(format!("resolved/{id}.toml"));

  let path = if active.exists() {
    active
  } else if resolved.exists() {
    log::debug!("reading resolved iteration {id}");
    resolved
  } else {
    return Err(Error::generic(format!("Iteration not found: '{id}'")));
  };

  log::trace!("reading iteration from {}", path.display());
  let content = fs::read_to_string(path)?;
  let iteration: Iteration = toml::from_str(&content)?;
  Ok(iteration)
}

/// Load all tasks referenced by an iteration, silently skipping any that
/// cannot be parsed or read.
pub fn read_iteration_tasks(layout: &DataLayout, iteration: &Iteration) -> Vec<Task> {
  let mut tasks = Vec::new();
  for task_ref in &iteration.tasks {
    let task_id_str = task_ref.strip_prefix("tasks/").unwrap_or(task_ref);
    if let Ok(task_id) = task_id_str.parse()
      && let Ok(task) = super::read_task(layout, &task_id)
    {
      tasks.push(task);
    }
  }
  tasks
}

/// Remove a task reference from an iteration.
pub fn remove_task(layout: &DataLayout, iteration_id: &Id, task_id: &str) -> super::Result<Iteration> {
  let mut iteration = read_iteration(layout, iteration_id)?;
  iteration.tasks.retain(|t| t != task_id);
  iteration.updated_at = Utc::now();
  write_iteration(layout, &iteration)?;
  Ok(iteration)
}

/// Move an iteration to the resolved directory, setting its `completed_at` timestamp.
pub fn resolve_iteration(layout: &DataLayout, id: &Id) -> super::Result<()> {
  let mut iteration = read_iteration(layout, id)?;
  let now = Utc::now();
  iteration.completed_at = Some(now);
  iteration.updated_at = now;

  let content = toml::to_string(&iteration)?;
  move_entity_file(
    layout,
    &content,
    &layout.iteration_dir().join(format!("resolved/{id}.toml")),
    &layout.iteration_dir().join(format!("{id}.toml")),
  )?;

  Ok(())
}

/// Resolve a short ID prefix to a full iteration [`Id`].
pub fn resolve_iteration_id(layout: &DataLayout, prefix: &str, include_resolved: bool) -> super::Result<Id> {
  log::debug!("resolving iteration ID prefix '{prefix}'");
  resolve_id(
    layout.iteration_dir(),
    Some(&layout.iteration_dir().join("resolved")),
    "toml",
    prefix,
    include_resolved,
    "Iteration",
  )
}

/// Apply a partial update to an existing iteration, moving it between active/resolved as needed.
pub fn update_iteration(layout: &DataLayout, id: &Id, patch: IterationPatch) -> super::Result<Iteration> {
  let mut iteration = read_iteration(layout, id)?;
  let was_resolved = is_iteration_resolved(layout, id);

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
      layout,
      &content,
      &layout.iteration_dir().join(format!("resolved/{id}.toml")),
      &layout.iteration_dir().join(format!("{id}.toml")),
    )?;
  } else if !iteration.status.is_terminal() && was_resolved {
    iteration.completed_at = None;
    let content = toml::to_string(&iteration)?;
    move_entity_file(
      layout,
      &content,
      &layout.iteration_dir().join(format!("{id}.toml")),
      &layout.iteration_dir().join(format!("resolved/{id}.toml")),
    )?;
  } else {
    write_iteration(layout, &iteration)?;
  }

  Ok(iteration)
}

/// Serialize and write an iteration to the active iterations directory.
pub fn write_iteration(layout: &DataLayout, iteration: &Iteration) -> super::Result<()> {
  ensure_dirs(layout)?;
  let path = layout.iteration_dir().join(format!("{}.toml", iteration.id));
  let content = toml::to_string(iteration)?;
  log::trace!("writing iteration {} to {}", iteration.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{
    config::storage::DataLayout,
    model::{Iteration, IterationFilter, iteration::Status},
  };

  fn make_layout(base: &std::path::Path) -> DataLayout {
    DataLayout::new(&crate::config::storage::Settings::default(), base)
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      let updated = crate::store::add_iteration_task(
        &make_layout(dir.path()),
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      crate::store::add_iteration_task(
        &make_layout(dir.path()),
        &iteration.id,
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk",
      )
      .unwrap();
      let updated = crate::store::add_iteration_task(
        &make_layout(dir.path()),
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
      crate::store::ensure_dirs(&make_layout(dir.path())).unwrap();

      let new = NewIteration {
        title: "Sprint 1".to_string(),
        ..Default::default()
      };

      let iteration = crate::store::create_iteration(&make_layout(dir.path()), new).unwrap();
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_layout(dir.path()), &iteration.id).unwrap();

      let filter = IterationFilter::default();
      let iterations = crate::store::list_iterations(&make_layout(dir.path()), &filter).unwrap();
      assert_eq!(iterations.len(), 0);
    }

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      let mut i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Failed");
      i2.status = Status::Failed;
      crate::store::write_iteration(&make_layout(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_layout(dir.path()), &i2).unwrap();

      let filter = IterationFilter {
        status: Some(Status::Failed),
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_layout(dir.path()), &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Failed");
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged");
      i1.tags = vec!["sprint".to_string()];
      let i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Untagged");
      crate::store::write_iteration(&make_layout(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_layout(dir.path()), &i2).unwrap();

      let filter = IterationFilter {
        tag: Some("sprint".to_string()),
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_layout(dir.path()), &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Tagged");
    }

    #[test]
    fn it_includes_resolved_when_all() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_layout(dir.path()), &iteration.id).unwrap();

      let filter = IterationFilter {
        all: true,
        ..Default::default()
      };
      let iterations = crate::store::list_iterations(&make_layout(dir.path()), &filter).unwrap();
      assert_eq!(iterations.len(), 1);
    }

    #[test]
    fn it_returns_active_iterations() {
      let dir = tempfile::tempdir().unwrap();
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "One");
      let i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Two");
      crate::store::write_iteration(&make_layout(dir.path()), &i1).unwrap();
      crate::store::write_iteration(&make_layout(dir.path()), &i2).unwrap();

      let filter = IterationFilter::default();
      let iterations = crate::store::list_iterations(&make_layout(dir.path()), &filter).unwrap();
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_layout(dir.path()), &iteration.id).unwrap();

      let loaded = crate::store::read_iteration(&make_layout(dir.path()), &iteration.id).unwrap();
      assert_eq!(loaded.title, "Test");
      assert!(loaded.completed_at.is_some());
    }

    #[test]
    fn it_roundtrips() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      let loaded = crate::store::read_iteration(&make_layout(dir.path()), &iteration.id).unwrap();
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      let updated = crate::store::remove_iteration_task(
        &make_layout(dir.path()),
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
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      let patch = IterationPatch {
        status: Some(Status::Completed),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_layout(dir.path()), &iteration.id, patch).unwrap();
      assert!(updated.completed_at.is_some());
      assert!(crate::store::is_iteration_resolved(
        &make_layout(dir.path()),
        &iteration.id
      ));
    }

    #[test]
    fn it_unresolves_on_active_status() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();
      crate::store::resolve_iteration(&make_layout(dir.path()), &iteration.id).unwrap();

      let patch = IterationPatch {
        status: Some(Status::Active),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_layout(dir.path()), &iteration.id, patch).unwrap();
      assert!(updated.completed_at.is_none());
      assert!(!crate::store::is_iteration_resolved(
        &make_layout(dir.path()),
        &iteration.id
      ));
    }

    #[test]
    fn it_updates_title() {
      let dir = tempfile::tempdir().unwrap();
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Old Title");
      crate::store::write_iteration(&make_layout(dir.path()), &iteration).unwrap();

      let patch = IterationPatch {
        title: Some("New Title".to_string()),
        ..Default::default()
      };

      let updated = crate::store::update_iteration(&make_layout(dir.path()), &iteration.id, patch).unwrap();
      assert_eq!(updated.title, "New Title");
    }
  }
}
