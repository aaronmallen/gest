mod get;
mod set;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Get or set custom metadata on a task.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Get a metadata value.
  Get(get::Command),
  /// Set a metadata value.
  Set(set::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Sub::Get(cmd) => cmd.call(context).await,
      Sub::Set(cmd) => cmd.call(context).await,
    }
  }
}
