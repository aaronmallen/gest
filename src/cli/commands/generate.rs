mod completions;
mod man_pages;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Generate shell completions and man pages.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Generate shell completions.
  Completions(completions::Command),
  /// Generate man pages.
  ManPages(man_pages::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("generate: entry");
    match &self.subcommand {
      Sub::Completions(cmd) => cmd.call(context).await,
      Sub::ManPages(cmd) => cmd.call(context).await,
    }
  }
}
