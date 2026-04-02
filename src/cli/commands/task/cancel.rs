use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::{Task, task::Status},
  store,
  ui::views::task::TaskUpdateView,
};

/// Cancel a task (shortcut for `task update <id> --status cancelled`).
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
}

impl Command {
  /// Mark the task as cancelled and print the confirmation view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(config, &self.id, true)?;

    let author = action::resolve_author(false)?;
    let task = action::set_status::<Task>(config, &id, Status::Cancelled, Some(&author))?;
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
