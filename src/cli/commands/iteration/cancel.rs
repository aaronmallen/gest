use clap::Args;

use crate::{
  action,
  cli::{self, AppContext},
  model::{Iteration, iteration::Status},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Cancel an iteration and all its non-terminal tasks (shortcut for `iteration update <id> --status cancelled`).
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Output as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Print only the iteration ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
}

impl Command {
  /// Mark the iteration as cancelled, cascading to all non-terminal tasks.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, false)?;

    let author = action::resolve_author(false)?;
    let iteration = action::set_status::<Iteration>(config, &id, Status::Cancelled, Some(&author))?;

    if self.json {
      let json = serde_json::to_string_pretty(&iteration)?;
      println!("{json}");
      return Ok(());
    }

    if self.quiet {
      println!("{}", iteration.id);
      return Ok(());
    }

    let msg = format!("Cancelled iteration {}", iteration.id);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}
