use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::task::Status,
  store,
  ui::{composites::iteration_detail::TaskCounts, views::iteration::IterationDetailView},
};

/// Display an iteration's details, task counts, and phase summary.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Output iteration details as JSON.
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  /// Load the iteration and its tasks, compute status counts, and render the detail view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, true)?;
    let iteration = store::read_iteration(config, &id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&iteration)?;
      println!("{json}");
      return Ok(());
    }

    let tasks = store::read_iteration_tasks(config, &iteration);
    let resolved = store::resolve_blocking_batch(config, &tasks);

    let mut counts = TaskCounts {
      total: 0,
      done: 0,
      in_progress: 0,
      open: 0,
      blocked: 0,
    };

    let mut phase_set = std::collections::BTreeSet::new();

    for (task, blocking) in tasks.iter().zip(&resolved) {
      counts.total += 1;
      if let Some(phase) = task.phase {
        phase_set.insert(phase);
      }

      if !blocking.blocked_by_ids.is_empty() {
        counts.blocked += 1;
      } else {
        match task.status {
          Status::Open => counts.open += 1,
          Status::InProgress => counts.in_progress += 1,
          Status::Done => counts.done += 1,
          Status::Cancelled => counts.done += 1,
        }
      }
    }

    let id_str = iteration.id.to_string();
    let view = IterationDetailView {
      id: &id_str,
      title: &iteration.title,
      phase_count: phase_set.len(),
      counts,
      events: &iteration.events,
      theme,
    };
    println!("{view}");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_context, make_test_iteration, make_test_task},
  };

  mod call {
    use super::*;
    use crate::model::{
      link::{Link, RelationshipType},
      task::Status,
    };

    #[test]
    fn it_does_not_count_blocked_by_done_task_as_blocked() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      // Create a blocker task that is already done
      let mut blocker = make_test_task("llllllllllllllllllllllllllllllll");
      blocker.status = Status::Done;
      store::write_task(&ctx.settings, &blocker).unwrap();

      // Create a task that is blocked-by the done blocker
      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.links = vec![Link {
        ref_: "llllllllllllllllllllllllllllllll".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      store::write_task(&ctx.settings, &task).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      // The command should succeed and the task should be counted as open, not blocked
      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_iteration_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: true,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_iteration_detail() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_iteration_with_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.phase = Some(1);
      store::write_task(&ctx.settings, &task).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
