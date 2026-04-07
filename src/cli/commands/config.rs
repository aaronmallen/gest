mod get;
mod set;
mod show;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// View or modify configuration.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Get a configuration value.
  Get(get::Command),
  /// Set a configuration value.
  Set(set::Command),
  /// Show the current configuration.
  Show(show::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Sub::Get(cmd) => cmd.call(context).await,
      Sub::Set(cmd) => cmd.call(context).await,
      Sub::Show(cmd) => cmd.call(context).await,
    }
  }
}
