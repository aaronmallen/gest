use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Add an existing task to an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Task ID or unique prefix to add.
  pub task_id: String,
}

impl Command {
  /// Resolve both IDs, then append the task reference to the iteration.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let iteration_id = store::resolve_iteration_id(layout, &self.id, false)?;
    let task_id = store::resolve_task_id(layout, &self.task_id, true)?;

    let task_ref = format!("tasks/{task_id}");
    store::add_iteration_task(layout, &iteration_id, &task_ref)?;

    let msg = format!("Added task {} to iteration {}", task_id, iteration_id);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_context, make_test_iteration, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_a_task_to_iteration() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        task_id: "kkkk".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(loaded.tasks, vec!["tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"]);
    }

    #[test]
    fn it_is_idempotent() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        task_id: "kkkk".to_string(),
      };
      cmd.call(&ctx).unwrap();
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(loaded.tasks.len(), 1);
    }
  }
}
