use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::views::iteration::IterationStatusView,
};

/// Display aggregated progress for an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Output iteration status as JSON.
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  /// Load the iteration, compute progress via `iteration_status`, and render.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, true)?;
    let iteration = store::read_iteration(config, &id)?;
    let progress = store::iteration_status(config, &id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&progress)?;
      println!("{json}");
      return Ok(());
    }

    let id_str = iteration.id.to_string();
    let view = IterationStatusView {
      id: &id_str,
      title: &iteration.title,
      status: iteration.status.as_str(),
      progress: &progress,
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
    model::task::Status as TaskStatus,
    store,
    test_helpers::{make_test_context, make_test_iteration, make_test_task},
  };

  mod call {
    use super::*;

    #[test]
    fn it_renders_status_card() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.phase = Some(1);
      task.status = TaskStatus::InProgress;
      task.assigned_to = Some("agent-1".to_string());
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

    #[test]
    fn it_renders_status_as_json() {
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
        json: true,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_renders_empty_iteration_status() {
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
  }
}
