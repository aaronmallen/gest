mod add;
mod list;
mod remove;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Manage tags across all entity types.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Option<Sub>,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Add tags to any entity (task, artifact, or iteration).
  Add(add::Command),
  /// List all tags.
  List(list::Command),
  /// Remove tags from any entity (task, artifact, or iteration).
  Remove(remove::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("tag: entry");
    match &self.subcommand {
      None => list::Command::default().call(context).await,
      Some(Sub::Add(cmd)) => cmd.call(context).await,
      Some(Sub::List(cmd)) => cmd.call(context).await,
      Some(Sub::Remove(cmd)) => cmd.call(context).await,
    }
  }

  /// Whether this command requires an initialized project.
  pub fn requires_project(&self) -> bool {
    match &self.subcommand {
      None | Some(Sub::List(_)) => false,
      Some(Sub::Add(_)) | Some(Sub::Remove(_)) => true,
    }
  }
}
