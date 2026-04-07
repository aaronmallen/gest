mod block;
mod cancel;
mod claim;
mod complete;
mod create;
mod link;
mod list;
mod meta;
mod note;
mod show;
mod tag;
mod untag;
mod update;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Manage tasks.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Mark a task as blocking another task.
  Block(block::Command),
  /// Cancel a task.
  Cancel(cancel::Command),
  /// Claim a task (assign and mark in-progress).
  Claim(claim::Command),
  /// Mark a task as done.
  Complete(complete::Command),
  /// Create a new task.
  Create(create::Command),
  /// Link a task to another entity.
  Link(link::Command),
  /// List tasks.
  List(list::Command),
  /// Get or set custom metadata.
  Meta(meta::Command),
  /// Manage notes on a task.
  Note(note::Command),
  /// Show a task.
  Show(show::Command),
  /// Add a tag to a task.
  Tag(tag::Command),
  /// Remove a tag from a task.
  Untag(untag::Command),
  /// Update a task.
  Update(update::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Sub::Block(cmd) => cmd.call(context).await,
      Sub::Cancel(cmd) => cmd.call(context).await,
      Sub::Claim(cmd) => cmd.call(context).await,
      Sub::Complete(cmd) => cmd.call(context).await,
      Sub::Create(cmd) => cmd.call(context).await,
      Sub::Link(cmd) => cmd.call(context).await,
      Sub::List(cmd) => cmd.call(context).await,
      Sub::Meta(cmd) => cmd.call(context).await,
      Sub::Note(cmd) => cmd.call(context).await,
      Sub::Show(cmd) => cmd.call(context).await,
      Sub::Tag(cmd) => cmd.call(context).await,
      Sub::Untag(cmd) => cmd.call(context).await,
      Sub::Update(cmd) => cmd.call(context).await,
    }
  }
}
