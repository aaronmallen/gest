//! Subcommands for reading and writing iteration metadata.

mod get;
mod set;
mod unset;

use clap::{Args, Subcommand};

use crate::{AppContext, actions, cli::Error, ui::json};

/// Read or write iteration metadata fields.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix (used when no subcommand is given).
  id: Option<String>,
  #[command(subcommand)]
  subcommand: Option<Sub>,
  #[command(flatten)]
  output: json::Flags,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Get a metadata value by dot-delimited path.
  Get(get::Command),
  /// Set a metadata value at a dot-delimited path.
  Set(set::Command),
  /// Remove a metadata value at a dot-delimited path.
  #[command(alias = "delete")]
  Unset(unset::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration meta: entry");
    match &self.subcommand {
      Some(Sub::Get(cmd)) => cmd.call(context).await,
      Some(Sub::Set(cmd)) => cmd.call(context).await,
      Some(Sub::Unset(cmd)) => cmd.call(context).await,
      None => {
        let id = self
          .id
          .as_deref()
          .ok_or_else(|| Error::Argument("<id> argument is required".to_string()))?;
        actions::meta::bare::<actions::Iteration>(context, id, &self.output).await
      }
    }
  }
}
