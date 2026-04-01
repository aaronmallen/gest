mod add;
mod delete;
mod list;
mod show;
mod update;

use clap::{Args, Subcommand};

use crate::cli::{self, AppContext};

/// Manage notes on a task.
#[derive(Debug, Args)]
pub struct Command {
  #[command(subcommand)]
  command: NoteCommand,
}

impl Command {
  /// Dispatch to the appropriate note subcommand.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    match &self.command {
      NoteCommand::Add(cmd) => cmd.call(ctx),
      NoteCommand::Delete(cmd) => cmd.call(ctx),
      NoteCommand::List(cmd) => cmd.call(ctx),
      NoteCommand::Show(cmd) => cmd.call(ctx),
      NoteCommand::Update(cmd) => cmd.call(ctx),
    }
  }
}

#[derive(Debug, Subcommand)]
enum NoteCommand {
  Add(add::Command),
  Delete(delete::Command),
  List(list::Command),
  Show(show::Command),
  Update(update::Command),
}
