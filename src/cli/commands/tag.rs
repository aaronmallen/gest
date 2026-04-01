//! Cross-entity tag management commands.

mod add;
mod list;
mod remove;

use clap::{Args, Subcommand};

use crate::cli::{self, AppContext};

/// Manage tags across tasks, artifacts, and iterations.
#[derive(Debug, Args)]
pub struct Command {
  #[command(subcommand)]
  command: TagCommand,
}

#[derive(Debug, Subcommand)]
enum TagCommand {
  Add(add::Command),
  List(list::Command),
  Remove(remove::Command),
}

impl Command {
  /// Route to the appropriate tag subcommand.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    match &self.command {
      TagCommand::Add(cmd) => cmd.call(ctx),
      TagCommand::List(cmd) => cmd.call(ctx),
      TagCommand::Remove(cmd) => cmd.call(ctx),
    }
  }
}
