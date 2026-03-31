use std::collections::BTreeMap;

use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::{
    composites::iteration_graph::{PhaseData, TaskData},
    views::iteration::IterationGraphView,
  },
};

/// Display the phased execution graph for an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
}

/// Intermediate representation of a task used to build graph view data.
struct TaskRow {
  blocked_by: Option<String>,
  id: String,
  is_blocking: bool,
  priority: Option<u8>,
  status: String,
  tags: Vec<String>,
  title: String,
}

impl Command {
  /// Load iteration tasks, group by phase, and render the execution graph.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(layout, &self.id, true)?;
    let iteration = store::read_iteration(layout, &id)?;

    let tasks = store::read_iteration_tasks(layout, &iteration);
    let resolved = store::resolve_blocking_batch(layout, &tasks);

    let mut phase_map: BTreeMap<u16, Vec<TaskRow>> = BTreeMap::new();

    for (task, blocking) in tasks.into_iter().zip(resolved) {
      let phase = task.phase.unwrap_or(0);
      let status_str = task.status.as_str();

      let blocked_by = blocking.blocked_by_ids.into_iter().next();
      let is_blocking = blocking.is_blocking;

      phase_map.entry(phase).or_default().push(TaskRow {
        blocked_by,
        id: task.id.to_string(),
        is_blocking,
        priority: task.priority,
        status: status_str.to_string(),
        tags: task.tags,
        title: task.title,
      });
    }

    let phases: Vec<PhaseData> = phase_map
      .iter()
      .enumerate()
      .map(|(idx, (_, tasks))| PhaseData {
        number: (idx + 1) as u32,
        name: None,
        tasks: tasks
          .iter()
          .map(|t| TaskData {
            status: &t.status,
            id: &t.id,
            title: &t.title,
            priority: t.priority,
            tags: &t.tags,
            is_blocking: t.is_blocking,
            blocked_by: t.blocked_by.as_deref(),
          })
          .collect(),
      })
      .collect();

    let view = IterationGraphView::new(&iteration.title, phases, theme);
    println!("{view}");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::task::Status,
    store,
    test_helpers::{make_test_context, make_test_iteration, make_test_task},
  };

  mod call {
    use super::*;

    #[test]
    fn it_renders_empty_graph() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_renders_graph_with_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut t1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.title = "First task".to_string();
      t1.phase = Some(1);
      t1.status = Status::Done;
      store::write_task(&ctx.layout, &t1).unwrap();

      let mut t2 = make_test_task("nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn");
      t2.title = "Second task".to_string();
      t2.phase = Some(2);
      t2.status = Status::Open;
      store::write_task(&ctx.layout, &t2).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec![
        "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        "tasks/nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn".to_string(),
      ];
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
