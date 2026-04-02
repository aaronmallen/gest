use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::{Task, task::Status},
  store,
  ui::views::task::TaskUpdateView,
};

/// Mark a task as done.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
}

impl Command {
  /// Set the task's status to done and print the confirmation view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(config, &self.id, true)?;

    let author = action::resolve_author(false)?;
    let task = action::set_status::<Task>(config, &id, Status::Done, Some(&author))?;
    let id_str = task.id.to_string();

    let view = TaskUpdateView {
      id: &id_str,
      fields: Vec::new(),
      status: Some(task.status.as_str()),
      theme,
    };
    println!("{view}");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_context, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_marks_task_as_done() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let updated = store::read_task(&ctx.settings, &task.id).unwrap();

      assert_eq!(updated.status, Status::Done);
    }
  }
}
