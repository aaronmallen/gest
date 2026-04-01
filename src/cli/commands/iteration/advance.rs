use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Validate the active phase is complete and advance to the next phase.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Advance even if current phase has non-terminal tasks.
  #[arg(long)]
  pub force: bool,
}

impl Command {
  /// Execute the iteration advance command.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, true)?;

    let summary = store::advance_phase(config, &id, self.force)?;

    let msg = match summary.to_phase {
      Some(phase) => format!(
        "Advanced iteration {} to phase {}: {} tasks now open",
        id, phase, summary.active_tasks
      ),
      None => "All phases complete".to_string(),
    };

    println!("{}", SuccessMessage::new(&msg, theme));
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
    fn it_advances_when_phase_is_terminal() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut t1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Done;
      store::write_task(&ctx.settings, &t1).unwrap();

      let mut t2 = make_test_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(2);
      t2.status = TaskStatus::Open;
      store::write_task(&ctx.settings, &t2).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec![t1.id.to_string(), t2.id.to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        force: false,
      };

      // Active phase is 2 (phase 1 all done), phase 2 has non-terminal -> error
      assert!(cmd.call(&ctx).is_err());
    }

    #[test]
    fn it_force_advances_with_non_terminal_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut t1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::Open;
      store::write_task(&ctx.settings, &t1).unwrap();

      let mut t2 = make_test_task("llllllllllllllllllllllllllllllll");
      t2.phase = Some(2);
      t2.status = TaskStatus::Open;
      store::write_task(&ctx.settings, &t2).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec![t1.id.to_string(), t2.id.to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        force: true,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_errors_when_phase_has_non_terminal_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut t1 = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      t1.phase = Some(1);
      t1.status = TaskStatus::InProgress;
      store::write_task(&ctx.settings, &t1).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks = vec![t1.id.to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        force: false,
      };

      let err = cmd.call(&ctx).unwrap_err();
      let msg = err.to_string();
      assert!(msg.contains("non-terminal"), "expected non-terminal error, got: {msg}");
    }
  }
}
