use std::{collections::HashMap, fs, path::Path};

use chrono::Utc;

use super::{
  fs::{ensure_dirs, move_entity_file, next_id, resolve_id},
  helpers::{load_entities_from_dirs, read_entity_file},
};
use crate::{
  config::Settings,
  model::{
    Id, NewTask, Task, TaskFilter, TaskPatch,
    event::{AuthorInfo, Event, EventKind},
    link::RelationshipType,
  },
};

/// Resolved blocking state for a task after checking referenced task statuses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedBlocking {
  /// IDs of non-terminal tasks that block this task.
  pub blocked_by_ids: Vec<String>,
  /// Whether this task actively blocks any other task (only true if this task is non-terminal).
  pub is_blocking: bool,
}

/// Persist a new task, resolving it immediately if the status is terminal.
pub fn create_task(config: &Settings, new: NewTask) -> super::Result<Task> {
  let now = Utc::now();
  let task = Task {
    assigned_to: new.assigned_to,
    created_at: now,
    description: new.description,
    events: Vec::new(),
    id: next_id(config)?,
    links: new.links,
    metadata: new.metadata,
    notes: Vec::new(),
    phase: new.phase,
    priority: new.priority,
    resolved_at: None,
    status: new.status,
    tags: new.tags,
    title: new.title,
    updated_at: now,
  };

  write_task(config, &task)?;

  if task.status.is_terminal() {
    resolve_task(config, &task.id)?;
    return read_task(config, &task.id);
  }

  Ok(task)
}

/// Check whether a task has been moved to the resolved directory.
pub fn is_task_resolved(config: &Settings, id: &Id) -> bool {
  let resolved_path = config.storage().task_dir().join(format!("resolved/{id}.toml"));
  let active_path = config.storage().task_dir().join(format!("{id}.toml"));
  resolved_path.exists() && !active_path.exists()
}

/// List tasks matching the given filter criteria.
pub fn list_tasks(config: &Settings, filter: &TaskFilter) -> super::Result<Vec<Task>> {
  let parse = |content: &str| Ok(toml::from_str::<Task>(content)?);
  let mut tasks = load_entities_from_dirs(
    config.storage().task_dir(),
    &config.storage().task_dir().join("resolved"),
    "toml",
    false,
    filter.all,
    parse,
  )?;

  tasks.retain(|task| {
    if let Some(ref assigned_to) = filter.assigned_to
      && task.assigned_to.as_deref() != Some(assigned_to.as_str())
    {
      return false;
    }
    if let Some(ref status) = filter.status
      && &task.status != status
    {
      return false;
    }
    if let Some(ref tag) = filter.tag
      && !task.tags.contains(tag)
    {
      return false;
    }
    true
  });

  Ok(tasks)
}

/// Load a single task by exact ID, checking both active and resolved directories.
pub fn read_task(config: &Settings, id: &Id) -> super::Result<Task> {
  let active = config.storage().task_dir().join(format!("{id}.toml"));
  let resolved = config.storage().task_dir().join(format!("resolved/{id}.toml"));

  read_entity_file(&active, &resolved, "resolved", "Task", id, |content| {
    Ok(toml::from_str::<Task>(content)?)
  })
}

/// Resolve the actual blocking state of a task by reading the status of referenced tasks.
///
/// A `blocked-by` link is only active if the referenced task exists and is non-terminal.
/// A `blocks` link is only active if the current task itself is non-terminal.
pub fn resolve_blocking(config: &Settings, task: &Task) -> ResolvedBlocking {
  let blocked_by_ids: Vec<String> = task
    .links
    .iter()
    .filter(|link| link.rel == RelationshipType::BlockedBy)
    .filter_map(|link| {
      let id_str = link.ref_.strip_prefix("tasks/")?;
      let id: Id = id_str.parse().ok()?;
      let referenced = read_task(config, &id).ok()?;
      if referenced.status.is_terminal() {
        None
      } else {
        Some(id_str.to_string())
      }
    })
    .collect();

  let is_blocking = !task.status.is_terminal() && task.links.iter().any(|link| link.rel == RelationshipType::Blocks);

  ResolvedBlocking {
    blocked_by_ids,
    is_blocking,
  }
}

/// Resolve blocking state for multiple tasks in a single batch.
///
/// Instead of N individual disk reads (one per `blocked-by` link), this collects all referenced
/// task IDs, reads each unique one once, and resolves blocking status from the in-memory set.
/// The returned vec is in the same order as `tasks`.
pub fn resolve_blocking_batch(config: &Settings, tasks: &[Task]) -> Vec<ResolvedBlocking> {
  // Collect all unique blocked-by task IDs across every task.
  let mut needed_ids: HashMap<String, Option<Task>> = HashMap::new();
  for task in tasks {
    for link in &task.links {
      if link.rel == RelationshipType::BlockedBy
        && let Some(id_str) = link.ref_.strip_prefix("tasks/")
      {
        needed_ids.entry(id_str.to_string()).or_insert(None);
      }
    }
  }

  // Read each unique referenced task exactly once.
  for (id_str, slot) in &mut needed_ids {
    if let Ok(id) = id_str.parse::<Id>()
      && let Ok(referenced) = read_task(config, &id)
    {
      *slot = Some(referenced);
    }
  }

  // Resolve blocking state per task from the in-memory map.
  tasks
    .iter()
    .map(|task| {
      let blocked_by_ids: Vec<String> = task
        .links
        .iter()
        .filter(|link| link.rel == RelationshipType::BlockedBy)
        .filter_map(|link| {
          let id_str = link.ref_.strip_prefix("tasks/")?;
          match needed_ids.get(id_str) {
            Some(Some(referenced)) if !referenced.status.is_terminal() => Some(id_str.to_string()),
            _ => None,
          }
        })
        .collect();

      let is_blocking =
        !task.status.is_terminal() && task.links.iter().any(|link| link.rel == RelationshipType::Blocks);

      ResolvedBlocking {
        blocked_by_ids,
        is_blocking,
      }
    })
    .collect()
}

/// Move a task to the resolved directory, setting its `resolved_at` timestamp.
pub fn resolve_task(config: &Settings, id: &Id) -> super::Result<()> {
  let mut task = read_task(config, id)?;
  let now = Utc::now();
  task.resolved_at = Some(now);
  task.updated_at = now;

  let content = toml::to_string(&task)?;
  move_entity_file(
    config,
    &content,
    &config.storage().task_dir().join(format!("resolved/{id}.toml")),
    &config.storage().task_dir().join(format!("{id}.toml")),
  )?;

  Ok(())
}

/// Resolve a short ID prefix to a full task [`Id`].
pub fn resolve_task_id(config: &Settings, prefix: &str, include_resolved: bool) -> super::Result<Id> {
  log::debug!("resolving task ID prefix '{prefix}'");
  resolve_id(
    config.storage().task_dir(),
    Some(&config.storage().task_dir().join("resolved")),
    "toml",
    prefix,
    include_resolved,
    "Task",
  )
}

/// Apply a partial update to an existing task, moving it between active/resolved as needed.
///
/// When `author` is provided, events are appended for detected status, phase, and priority changes.
pub fn update_task(config: &Settings, id: &Id, patch: TaskPatch, author: Option<&AuthorInfo>) -> super::Result<Task> {
  let mut task = read_task(config, id)?;
  let was_resolved = is_task_resolved(config, id);

  if let Some(author) = author {
    let now = Utc::now();

    if let Some(ref new_status) = patch.status
      && *new_status != task.status
    {
      task.events.push(Event {
        author: author.author.clone(),
        author_email: author.author_email.clone(),
        author_type: author.author_type.clone(),
        created_at: now,
        description: None,
        id: Id::new(),
        kind: EventKind::StatusChange {
          from: task.status.as_str().to_string(),
          to: new_status.as_str().to_string(),
        },
      });
    }

    if let Some(ref new_phase) = patch.phase
      && *new_phase != task.phase
    {
      task.events.push(Event {
        author: author.author.clone(),
        author_email: author.author_email.clone(),
        author_type: author.author_type.clone(),
        created_at: now,
        description: None,
        id: Id::new(),
        kind: EventKind::PhaseChange {
          from: task.phase,
          to: *new_phase,
        },
      });
    }

    if let Some(ref new_priority) = patch.priority
      && *new_priority != task.priority
    {
      task.events.push(Event {
        author: author.author.clone(),
        author_email: author.author_email.clone(),
        author_type: author.author_type.clone(),
        created_at: now,
        description: None,
        id: Id::new(),
        kind: EventKind::PriorityChange {
          from: task.priority,
          to: *new_priority,
        },
      });
    }
  }

  if let Some(assigned_to) = patch.assigned_to {
    task.assigned_to = assigned_to;
  }
  if let Some(description) = patch.description {
    task.description = description;
  }
  if let Some(metadata) = patch.metadata {
    task.metadata = metadata;
  }
  if let Some(phase) = patch.phase {
    task.phase = phase;
  }
  if let Some(priority) = patch.priority {
    task.priority = priority;
  }
  if let Some(status) = patch.status {
    task.status = status;
  }
  if let Some(tags) = patch.tags {
    task.tags = tags;
  }
  if let Some(title) = patch.title {
    task.title = title;
  }

  task.updated_at = Utc::now();

  if task.status.is_terminal() && !was_resolved {
    task.resolved_at = Some(task.updated_at);
    let content = toml::to_string(&task)?;
    move_entity_file(
      config,
      &content,
      &config.storage().task_dir().join(format!("resolved/{id}.toml")),
      &config.storage().task_dir().join(format!("{id}.toml")),
    )?;
  } else if !task.status.is_terminal() && was_resolved {
    task.resolved_at = None;
    let content = toml::to_string(&task)?;
    move_entity_file(
      config,
      &content,
      &config.storage().task_dir().join(format!("{id}.toml")),
      &config.storage().task_dir().join(format!("resolved/{id}.toml")),
    )?;
  } else {
    write_task(config, &task)?;
  }

  Ok(task)
}

/// Serialize and write a task, respecting its current location on disk.
///
/// If the task already exists in the resolved directory, it is written there
/// to avoid creating a duplicate in the active directory.
pub fn write_task(config: &Settings, task: &Task) -> super::Result<()> {
  let resolved_path = config.storage().task_dir().join(format!("resolved/{}.toml", task.id));
  let path = if resolved_path.exists() {
    resolved_path
  } else {
    config.storage().task_dir().join(format!("{}.toml", task.id))
  };
  write_task_to_path(config, task, &path)
}

/// Inner helper that writes a task to an explicit path.
fn write_task_to_path(config: &Settings, task: &Task, path: &Path) -> super::Result<()> {
  ensure_dirs(config)?;
  let content = toml::to_string(task)?;
  log::trace!("writing task {} to {}", task.id, path.display());
  fs::write(path, content)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{
    config::Settings,
    model::{Task, TaskFilter, link::Link, task::Status},
  };

  fn make_config(base: &std::path::Path) -> Settings {
    crate::test_helpers::make_test_config(base.to_path_buf())
  }

  fn make_test_task(id: &str, title: &str) -> Task {
    Task {
      title: title.to_string(),
      ..crate::test_helpers::make_test_task(id)
    }
  }

  mod is_task_resolved {
    use super::*;

    #[test]
    fn it_returns_false_for_active_task() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      assert!(!crate::store::is_task_resolved(&make_config(dir.path()), &task.id));
    }

    #[test]
    fn it_returns_true_for_resolved_task() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      assert!(crate::store::is_task_resolved(&make_config(dir.path()), &task.id));
    }
  }

  mod list_tasks {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_excludes_resolved_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active Task");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let filter = TaskFilter::default();
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn it_excludes_unassigned_when_filtering_by_assigned_to() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Unassigned");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let filter = TaskFilter {
        assigned_to: Some("agent-1".to_string()),
        ..Default::default()
      };
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn it_filters_by_assigned_to() {
      let dir = tempfile::tempdir().unwrap();
      let mut task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Assigned Task");
      task1.assigned_to = Some("agent-1".to_string());
      let mut task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Other Task");
      task2.assigned_to = Some("agent-2".to_string());
      let task3 = make_test_task("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Unassigned Task");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task3).unwrap();

      let filter = TaskFilter {
        assigned_to: Some("agent-1".to_string()),
        ..Default::default()
      };
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Assigned Task");
    }

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let mut task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Open Task");
      task1.status = Status::Open;
      let mut task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Done Task");
      task2.status = Status::Done;
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();

      let filter = TaskFilter {
        status: Some(Status::Done),
        ..Default::default()
      };
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Done Task");
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let mut task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged");
      task1.tags = vec!["rust".to_string()];
      let task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Untagged");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();

      let filter = TaskFilter {
        tag: Some("rust".to_string()),
        ..Default::default()
      };
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Tagged");
    }

    #[test]
    fn it_includes_resolved_when_all() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active Task");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let filter = TaskFilter {
        all: true,
        ..Default::default()
      };
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn it_returns_active_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task One");
      let task2 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Task Two");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();

      let filter = TaskFilter::default();
      let tasks = crate::store::list_tasks(&make_config(dir.path()), &filter).unwrap();
      assert_eq!(tasks.len(), 2);
    }
  }

  mod resolve_blocking {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::link::RelationshipType;

    #[test]
    fn it_excludes_cancelled_blocker() {
      let dir = tempfile::tempdir().unwrap();
      let mut blocker = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Cancelled Blocker");
      blocker.status = Status::Cancelled;
      crate::store::write_task(&make_config(dir.path()), &blocker).unwrap();

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(result.blocked_by_ids.is_empty());
    }

    #[test]
    fn it_excludes_done_blocker() {
      let dir = tempfile::tempdir().unwrap();
      let mut blocker = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Done Blocker");
      blocker.status = Status::Done;
      crate::store::write_task(&make_config(dir.path()), &blocker).unwrap();

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(result.blocked_by_ids.is_empty());
    }

    #[test]
    fn it_excludes_missing_blocker() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(&make_config(dir.path())).unwrap();

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(result.blocked_by_ids.is_empty());
    }

    #[test]
    fn it_hides_is_blocking_for_cancelled_task() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Cancelled Blocker");
      task.status = Status::Cancelled;
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::Blocks,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(!result.is_blocking);
    }

    #[test]
    fn it_hides_is_blocking_for_done_task() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Done Blocker");
      task.status = Status::Done;
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::Blocks,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(!result.is_blocking);
    }

    #[test]
    fn it_resolves_blocker_in_resolved_directory() {
      let dir = tempfile::tempdir().unwrap();
      let mut blocker = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Resolved Blocker");
      blocker.status = Status::Done;
      crate::store::write_task(&make_config(dir.path()), &blocker).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &blocker.id).unwrap();

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(result.blocked_by_ids.is_empty());
    }

    #[test]
    fn it_returns_active_blocker() {
      let dir = tempfile::tempdir().unwrap();
      let blocker = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Blocker");
      crate::store::write_task(&make_config(dir.path()), &blocker).unwrap();

      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);
      assert_eq!(result.blocked_by_ids, vec!["kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]);
      assert!(!result.is_blocking);
    }

    #[test]
    fn it_shows_is_blocking_for_non_terminal_task_with_blocks_link() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocking");
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::Blocks,
      }];
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let result = crate::store::resolve_blocking(&make_config(dir.path()), &task);

      assert!(result.is_blocking);
    }
  }

  mod resolve_task {
    use super::*;

    #[test]
    fn it_moves_file_to_resolved() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Resolve");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
    }

    #[test]
    fn it_sets_resolved_at() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "To Resolve");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let loaded = crate::store::read_task(&make_config(dir.path()), &task.id).unwrap();
      assert!(loaded.resolved_at.is_some());
    }
  }

  mod resolve_task_id {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_ambiguous_for_multiple_active_without_checking_resolved() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Active 2");
      let task3 = make_test_task("zyxwmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Resolved");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task3).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task3.id).unwrap();

      let result = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", true);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
      assert!(
        !err.contains("zyxwmmmmmmmmmmmmmmmmmmmmmmmmmmmm"),
        "Should not include archived ID in error: {err}"
      );
    }

    #[test]
    fn it_errors_ambiguous_for_multiple_resolved_matches() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Resolved 2");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task1.id).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task2.id).unwrap();

      let result = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", true);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
    }

    #[test]
    fn it_errors_not_found_for_resolved_when_not_included() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let result = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected --all hint, got: {err}");
    }

    #[test]
    fn it_errors_on_ambiguous_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let task1 = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task 1");
      let task2 = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Task 2");
      crate::store::write_task(&make_config(dir.path()), &task1).unwrap();
      crate::store::write_task(&make_config(dir.path()), &task2).unwrap();

      let result = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("Ambiguous"), "Expected ambiguous error, got: {err}");
    }

    #[test]
    fn it_errors_on_not_found() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(&make_config(dir.path())).unwrap();

      let result = crate::store::resolve_task_id(&make_config(dir.path()), "nnnn", false);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("not found"), "Expected not found error, got: {err}");
      assert!(err.contains("--all"), "Expected --all hint, got: {err}");
    }

    #[test]
    fn it_falls_back_to_resolved_when_no_active_match() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let resolved = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_prefers_active_over_resolved_with_shared_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let active = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Active");
      let to_resolve = make_test_task("zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Resolved");
      crate::store::write_task(&make_config(dir.path()), &active).unwrap();
      crate::store::write_task(&make_config(dir.path()), &to_resolve).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &to_resolve.id).unwrap();

      let resolved = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", true).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_resolves_unique_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let resolved = crate::store::resolve_task_id(&make_config(dir.path()), "zyxw", false).unwrap();
      assert_eq!(resolved.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }
  }

  mod task_io {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_task_with_links_and_metadata() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test Task");
      task.description = "A description".to_string();
      task.links = vec![Link {
        ref_: "https://example.com".to_string(),
        rel: crate::model::link::RelationshipType::RelatesTo,
      }];
      task.metadata = {
        let mut table = toml::Table::new();
        table.insert("priority".to_string(), toml::Value::String("high".to_string()));
        table
      };
      task.tags = vec!["rust".to_string(), "test".to_string()];

      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      let loaded = crate::store::read_task(&make_config(dir.path()), &task.id).unwrap();

      assert_eq!(task, loaded);
    }
  }

  mod update_task {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::model::TaskPatch;

    #[test]
    fn it_keeps_active_task_in_active_dir() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Original");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        title: Some("Updated".to_string()),
        ..Default::default()
      };
      crate::store::update_task(&make_config(dir.path()), &task.id, patch, None).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        !dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
    }

    #[test]
    fn it_writes_to_resolved_not_active() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Original");
      task.status = Status::Done;
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();
      crate::store::resolve_task(&make_config(dir.path()), &task.id).unwrap();

      let patch = TaskPatch {
        title: Some("Updated".to_string()),
        ..Default::default()
      };
      crate::store::update_task(&make_config(dir.path()), &task.id, patch, None).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );

      let loaded = crate::store::read_task(&make_config(dir.path()), &task.id).unwrap();
      assert_eq!(loaded.title, "Updated");
    }
  }

  mod update_task_events {
    use super::*;
    use crate::model::{
      TaskPatch,
      event::{AuthorInfo, EventKind},
      note::AuthorType,
    };

    fn make_author() -> AuthorInfo {
      AuthorInfo {
        author: "alice".to_string(),
        author_email: Some("alice@example.com".to_string()),
        author_type: AuthorType::Human,
      }
    }

    #[test]
    fn it_appends_phase_change_event() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      task.phase = Some(1);
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        phase: Some(Some(2)),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, Some(&make_author())).unwrap();

      assert_eq!(updated.events.len(), 1);
      assert!(matches!(
        &updated.events[0].kind,
        EventKind::PhaseChange {
          from: Some(1),
          to: Some(2)
        }
      ));
    }

    #[test]
    fn it_appends_priority_change_event() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      task.priority = Some(0);
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        priority: Some(Some(1)),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, Some(&make_author())).unwrap();

      assert_eq!(updated.events.len(), 1);
      assert!(matches!(
        &updated.events[0].kind,
        EventKind::PriorityChange {
          from: Some(0),
          to: Some(1)
        }
      ));
    }

    #[test]
    fn it_appends_status_change_event() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        status: Some(Status::InProgress),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, Some(&make_author())).unwrap();

      assert_eq!(updated.events.len(), 1);
      assert!(matches!(
        &updated.events[0].kind,
        EventKind::StatusChange { from, to } if from == "open" && to == "in-progress"
      ));
      assert_eq!(updated.events[0].author, "alice");
    }

    #[test]
    fn it_generates_multiple_events_for_multiple_changes() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        phase: Some(Some(2)),
        priority: Some(Some(1)),
        status: Some(Status::InProgress),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, Some(&make_author())).unwrap();

      assert_eq!(updated.events.len(), 3);
    }

    #[test]
    fn it_generates_no_events_for_same_status() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        status: Some(Status::Open),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, Some(&make_author())).unwrap();

      assert!(updated.events.is_empty());
    }

    #[test]
    fn it_generates_no_events_without_author() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Test");
      crate::store::write_task(&make_config(dir.path()), &task).unwrap();

      let patch = TaskPatch {
        status: Some(Status::InProgress),
        ..Default::default()
      };
      let updated = crate::store::update_task(&make_config(dir.path()), &task.id, patch, None).unwrap();

      assert!(updated.events.is_empty());
    }
  }
}
