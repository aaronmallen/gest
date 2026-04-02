//! Subcommands for generating shell completions and man pages.

mod completions;
mod man_pages;

use clap::{Args, Subcommand};

use crate::cli::{self, AppContext};

/// Entry point for the `generate` subcommand tree.
#[derive(Debug, Args)]
pub struct Command {
  #[command(subcommand)]
  command: GenerateCommand,
}

#[derive(Debug, Subcommand)]
enum GenerateCommand {
  Completions(completions::Command),
  ManPages(man_pages::Command),
}

impl Command {
  /// Dispatch to the appropriate generate subcommand.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    match &self.command {
      GenerateCommand::Completions(cmd) => cmd.call(),
      GenerateCommand::ManPages(cmd) => cmd.call(ctx),
    }
  }
}
