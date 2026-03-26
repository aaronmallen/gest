mod completions;
mod man_pages;

use clap::{Args, Subcommand};

use crate::Result;

/// Generate shell completions and man pages
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
  pub fn call(&self) -> Result<()> {
    match &self.command {
      GenerateCommand::Completions(cmd) => cmd.call(),
      GenerateCommand::ManPages(cmd) => cmd.call(),
    }
  }
}
