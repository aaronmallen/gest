mod add;
mod delete;
mod list;
mod show;
mod update;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Manage notes on an artifact.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Add a note to an artifact.
  Add(add::Command),
  /// Delete a note from an artifact.
  Delete(delete::Command),
  /// List notes on an artifact.
  List(list::Command),
  /// Show a single note.
  Show(show::Command),
  /// Update a note's body.
  Update(update::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Sub::Add(cmd) => cmd.call(context).await,
      Sub::Delete(cmd) => cmd.call(context).await,
      Sub::List(cmd) => cmd.call(context).await,
      Sub::Show(cmd) => cmd.call(context).await,
      Sub::Update(cmd) => cmd.call(context).await,
    }
  }
}
