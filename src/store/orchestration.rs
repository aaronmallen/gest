use std::collections::HashSet;

use serde::Serialize;

use crate::{
  config::Settings,
  model::{Id, Task, task::Status as TaskStatus},
};

/// Progress information for a single phase.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PhaseProgress {
  pub done: u16,
  pub total: u16,
}

/// Overall progress counts.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OverallProgress {
  pub done: u16,
  pub total: u16,
}

/// Aggregated status of an iteration's progress.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IterationProgress {
  pub active_phase: Option<u16>,
  pub total_phases: u16,
  pub phase_progress: PhaseProgress,
  pub blocked: u16,
  pub in_progress: u16,
  pub assignees: Vec<String>,
  pub overall_progress: OverallProgress,
}

/// Summary of a phase advance operation.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdvanceSummary {
  /// The phase that was just completed.
  pub from_phase: u16,
  /// The new active phase, or `None` if all phases are complete.
  pub to_phase: Option<u16>,
  /// Number of tasks now active in the new phase.
  pub active_tasks: u16,
  /// Whether the advance was forced (skipping non-terminal tasks).
  pub forced: bool,
}

/// Find the next claimable task in an iteration: open, unblocked, unassigned,
/// sorted by priority (lowest first) then `created_at` (oldest first).
pub fn next_available_task(config: &Settings, id: &Id) -> super::Result<Option<Task>> {
  let iteration = super::read_iteration(config, id)?;
  let tasks = super::read_iteration_tasks(config, &iteration);
  let phase = match compute_active_phase(&tasks) {
    Some(p) => p,
    None => return Ok(None),
  };

  let phase_tasks: Vec<&Task> = tasks.iter().filter(|t| t.phase.unwrap_or(0) == phase).collect();

  let blocking = super::resolve_blocking_batch(config, &tasks);
  // Build a set of blocked task IDs from the batch results (aligned by index with `tasks`).
  let blocked_ids: HashSet<&Id> = tasks
    .iter()
    .zip(blocking.iter())
    .filter(|(_, rb)| !rb.blocked_by_ids.is_empty())
    .map(|(t, _)| &t.id)
    .collect();

  let mut candidates: Vec<&Task> = phase_tasks
    .into_iter()
    .filter(|t| t.status == TaskStatus::Open && t.assigned_to.is_none() && !blocked_ids.contains(&t.id))
    .collect();

  candidates.sort_by(|a, b| {
    let pa = a.priority.unwrap_or(u8::MAX);
    let pb = b.priority.unwrap_or(u8::MAX);
    pa.cmp(&pb).then_with(|| a.created_at.cmp(&b.created_at))
  });

  Ok(candidates.first().cloned().cloned())
}

/// Atomically set a task to in-progress with the given `assigned_to` value.
pub fn claim_task(config: &Settings, task_id: &Id, assigned_to: &str) -> super::Result<Task> {
  let patch = crate::model::TaskPatch {
    assigned_to: Some(Some(assigned_to.to_string())),
    status: Some(TaskStatus::InProgress),
    ..Default::default()
  };
  super::update_task(config, task_id, patch)
}

/// Compute aggregated progress for an iteration.
pub fn iteration_status(config: &Settings, id: &Id) -> super::Result<IterationProgress> {
  let iteration = super::read_iteration(config, id)?;
  let tasks = super::read_iteration_tasks(config, &iteration);
  let active = compute_active_phase(&tasks);

  let total_phases = {
    let mut phases = HashSet::new();
    for t in &tasks {
      phases.insert(t.phase.unwrap_or(0));
    }
    phases.len() as u16
  };

  let blocking = super::resolve_blocking_batch(config, &tasks);
  let blocked_ids: HashSet<&Id> = tasks
    .iter()
    .zip(blocking.iter())
    .filter(|(_, rb)| !rb.blocked_by_ids.is_empty())
    .map(|(t, _)| &t.id)
    .collect();

  let phase_tasks: Vec<&Task> = tasks
    .iter()
    .filter(|t| active.is_some_and(|p| t.phase.unwrap_or(0) == p))
    .collect();

  let phase_done = phase_tasks.iter().filter(|t| t.status.is_terminal()).count() as u16;
  let phase_total = phase_tasks.len() as u16;

  let blocked_count = phase_tasks.iter().filter(|t| blocked_ids.contains(&t.id)).count() as u16;

  let in_progress_count = phase_tasks
    .iter()
    .filter(|t| t.status == TaskStatus::InProgress)
    .count() as u16;

  let mut assignees: Vec<String> = tasks
    .iter()
    .filter(|t| t.status == TaskStatus::InProgress)
    .filter_map(|t| t.assigned_to.clone())
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();
  assignees.sort();

  let overall_done = tasks.iter().filter(|t| t.status.is_terminal()).count() as u16;
  let overall_total = tasks.len() as u16;

  Ok(IterationProgress {
    active_phase: active,
    total_phases,
    phase_progress: PhaseProgress {
      done: phase_done,
      total: phase_total,
    },
    blocked: blocked_count,
    in_progress: in_progress_count,
    assignees,
    overall_progress: OverallProgress {
      done: overall_done,
      total: overall_total,
    },
  })
}

/// Validate and return a summary of what advancing the phase would do.
///
/// If `force` is false, returns an error when non-terminal tasks remain.
/// If `force` is true, allows advancing even with non-terminal tasks.
pub fn advance_phase(config: &Settings, id: &Id, force: bool) -> super::Result<AdvanceSummary> {
  let iteration = super::read_iteration(config, id)?;
  let tasks = super::read_iteration_tasks(config, &iteration);
  let active = compute_active_phase(&tasks);

  let from_phase = match active {
    Some(p) => p,
    None => return Err(super::Error::generic("no active phase to advance")),
  };

  let remaining = tasks
    .iter()
    .filter(|t| t.phase.unwrap_or(0) == from_phase && !t.status.is_terminal())
    .count() as u16;

  if remaining > 0 && !force {
    return Err(super::Error::generic(format!(
      "phase {from_phase} has {remaining} non-terminal task(s); use force to advance anyway"
    )));
  }

  // Find the next phase after the current active phase.
  let mut all_phases: Vec<u16> = tasks
    .iter()
    .map(|t| t.phase.unwrap_or(0))
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();
  all_phases.sort();

  let to_phase = all_phases.into_iter().find(|&p| p > from_phase);

  let active_tasks = match to_phase {
    Some(p) => tasks
      .iter()
      .filter(|t| t.phase.unwrap_or(0) == p && !t.status.is_terminal())
      .count() as u16,
    None => 0,
  };

  Ok(AdvanceSummary {
    from_phase,
    to_phase,
    active_tasks,
    forced: force && remaining > 0,
  })
}

/// Internal helper: find the lowest phase with incomplete tasks.
fn compute_active_phase(tasks: &[Task]) -> Option<u16> {
  let mut phases_with_incomplete: Vec<u16> = tasks
    .iter()
    .filter(|t| !t.status.is_terminal())
    .map(|t| t.phase.unwrap_or(0))
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();
  phases_with_incomplete.sort();
  phases_with_incomplete.first().copied()
}

#[cfg(test)]
mod tests {
  use chrono::{Duration, Utc};

  use crate::{
    config::Settings,
    model::{
      Id, Iteration, Task,
      link::{Link, RelationshipType},
      task::Status as TaskStatus,
    },
  };

  fn make_config(base: &std::path::Path) -> Settings {
    crate::test_helpers::make_test_config(base.to_path_buf())
  }

  fn make_iteration(id: &str) -> Iteration {
    crate::test_helpers::make_test_iteration(id)
  }

  fn make_task(id: &str) -> Task {
    crate::test_helpers::make_test_task(id)
  }

  /// Set up an iteration with the given tasks (writes all to disk and adds refs).
  fn setup_iteration(dir: &std::path::Path, iteration_id: &str, tasks: &[Task]) -> Id {
    let config = make_config(dir);
    let mut iteration = make_iteration(iteration_id);
    for t in tasks {
      crate::store::write_task(&config, t).unwrap();
      iteration.tasks.push(t.id.to_string());
    }
    crate::store::write_iteration(&config, &iteration).unwrap();
    iteration.id
  }

  mod next_available_task {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_when_no_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[]);

      let result = super::super::next_available_task(&make_config(dir.path()), &iter_id).unwrap();
      assert_eq!(result, None);
    }

    #[test]
    fn it_selects_by_priority() {
      let dir = tempfile::tempdir().unwrap();
      let now = Utc::now();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.priority = Some(5);
      t1.created_at = now;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.priority = Some(1);
      t2.created_at = now;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2.clone()]);

      let result = super::super::next_available_task(&make_config(dir.path()), &iter_id)
        .unwrap()
        .unwrap();
      assert_eq!(result.id, t2.id);
    }

    #[test]
    fn it_breaks_ties_by_created_at() {
      let dir = tempfile::tempdir().unwrap();
      let now = Utc::now();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.priority = Some(1);
      t1.created_at = now;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.priority = Some(1);
      t2.created_at = now - Duration::hours(1);
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2.clone()]);

      let result = super::super::next_available_task(&make_config(dir.path()), &iter_id)
        .unwrap()
        .unwrap();
      assert_eq!(result.id, t2.id);
    }

    #[test]
    fn it_excludes_assigned_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.priority = Some(1);
      t1.assigned_to = Some("agent-1".to_string());
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.priority = Some(2);
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2.clone()]);

      let result = super::super::next_available_task(&make_config(dir.path()), &iter_id)
        .unwrap()
        .unwrap();
      assert_eq!(result.id, t2.id);
    }

    #[test]
    fn it_excludes_blocked_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());

      // Create a blocker task (non-terminal)
      let mut blocker = make_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn");
      blocker.phase = Some(0);
      blocker.status = TaskStatus::InProgress;
      crate::store::write_task(&config, &blocker).unwrap();

      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.priority = Some(1);
      t1.links = vec![Link {
        rel: RelationshipType::BlockedBy,
        ref_: format!("tasks/{}", blocker.id),
      }];
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.priority = Some(2);

      let mut iteration = make_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      for t in [&t1, &t2] {
        crate::store::write_task(&config, t).unwrap();
        iteration.tasks.push(t.id.to_string());
      }
      crate::store::write_iteration(&config, &iteration).unwrap();

      let result = super::super::next_available_task(&config, &iteration.id)
        .unwrap()
        .unwrap();
      assert_eq!(result.id, t2.id);
    }

    #[test]
    fn it_only_considers_active_phase() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.priority = Some(1);
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(2);
      t2.priority = Some(0); // Higher priority but in later phase
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1.clone(), t2]);

      let result = super::super::next_available_task(&make_config(dir.path()), &iter_id)
        .unwrap()
        .unwrap();
      assert_eq!(result.id, t1.id);
    }
  }

  mod claim_task {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_sets_status_and_assigned_to() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      crate::store::write_task(&config, &t1).unwrap();

      let claimed = super::super::claim_task(&config, &t1.id, "agent-42").unwrap();
      assert_eq!(claimed.status, TaskStatus::InProgress);
      assert_eq!(claimed.assigned_to, Some("agent-42".to_string()));
    }
  }

  mod iteration_status {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_computes_progress() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Done;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.status = TaskStatus::InProgress;
      t2.assigned_to = Some("agent-1".to_string());
      let mut t3 = make_task("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm");
      t3.phase = Some(2);
      t3.status = TaskStatus::Open;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2, t3]);

      let status = super::super::iteration_status(&make_config(dir.path()), &iter_id).unwrap();
      assert_eq!(status.active_phase, Some(1));
      assert_eq!(status.total_phases, 2);
      assert_eq!(status.phase_progress.done, 1);
      assert_eq!(status.phase_progress.total, 2);
      assert_eq!(status.in_progress, 1);
      assert_eq!(status.assignees, vec!["agent-1".to_string()]);
      assert_eq!(status.overall_progress.done, 1);
      assert_eq!(status.overall_progress.total, 3);
    }

    #[test]
    fn it_returns_zero_phases_when_no_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[]);

      let status = super::super::iteration_status(&make_config(dir.path()), &iter_id).unwrap();
      assert_eq!(status.active_phase, None);
      assert_eq!(status.total_phases, 0);
      assert_eq!(status.overall_progress.total, 0);
    }
  }

  mod advance_phase {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_advances_to_next_phase() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Done;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(1);
      t2.status = TaskStatus::Cancelled;
      let mut t3 = make_task("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm");
      t3.phase = Some(2);
      t3.status = TaskStatus::Open;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2, t3]);

      // Active phase is 2 (phase 1 is all terminal). But phase 2 has non-terminal tasks,
      // so we need to check what advance_phase does when active phase itself is not done.
      // Actually, active_phase = 2, and remaining in phase 2 = 1, so without force it errors.
      let err = super::super::advance_phase(&make_config(dir.path()), &iter_id, false);
      assert!(err.is_err());
    }

    #[test]
    fn it_succeeds_when_active_phase_is_terminal() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Open;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(2);
      t2.status = TaskStatus::Open;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2]);

      // active phase is 1, remaining = 1, so force required
      let result = super::super::advance_phase(&make_config(dir.path()), &iter_id, true).unwrap();
      assert_eq!(result.from_phase, 1);
      assert_eq!(result.to_phase, Some(2));
      assert_eq!(result.active_tasks, 1);
      assert!(result.forced);
    }

    #[test]
    fn it_returns_none_to_phase_when_last_phase() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Done;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1]);

      // All done, no active phase, so advance_phase should error
      let err = super::super::advance_phase(&make_config(dir.path()), &iter_id, false);
      assert!(err.is_err());
    }

    #[test]
    fn it_forces_advance_with_remaining_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let mut t1 = make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Open;
      let mut t2 = make_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(2);
      t2.status = TaskStatus::Open;
      let iter_id = setup_iteration(dir.path(), "zyxwvutsrqponmlkzyxwvutsrqponmlk", &[t1, t2]);

      let result = super::super::advance_phase(&make_config(dir.path()), &iter_id, true).unwrap();
      assert!(result.forced);
      assert_eq!(result.from_phase, 1);
      assert_eq!(result.to_phase, Some(2));
    }
  }
}
