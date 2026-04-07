mod add;
mod advance;
mod cancel;
mod complete;
mod create;
mod graph;
mod link;
mod list;
mod meta;
mod next;
mod remove;
mod reopen;
mod show;
mod status;
mod tag;
mod untag;
mod update;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error};

/// Manage iterations.
#[derive(Args, Debug)]
pub struct Command {
  #[command(subcommand)]
  subcommand: Sub,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Add a task to an iteration.
  Add(add::Command),
  /// Validate active phase and advance to the next phase.
  Advance(advance::Command),
  /// Cancel an iteration.
  Cancel(cancel::Command),
  /// Complete an iteration.
  Complete(complete::Command),
  /// Create a new iteration.
  #[command(visible_alias = "new")]
  Create(create::Command),
  /// Show phased task dependency graph.
  Graph(graph::Command),
  /// Link an iteration to another entity.
  Link(link::Command),
  /// List iterations.
  #[command(visible_alias = "ls")]
  List(list::Command),
  /// Get or set custom metadata.
  Meta(meta::Command),
  /// Return the next available task in an iteration.
  Next(next::Command),
  /// Remove a task from an iteration.
  #[command(visible_alias = "rm")]
  Remove(remove::Command),
  /// Reopen a completed or cancelled iteration.
  Reopen(reopen::Command),
  /// Show an iteration.
  #[command(visible_alias = "view")]
  Show(show::Command),
  /// Show iteration progress summary.
  Status(status::Command),
  /// Add a tag to an iteration.
  Tag(tag::Command),
  /// Remove a tag from an iteration.
  Untag(untag::Command),
  /// Update an iteration.
  #[command(visible_alias = "edit")]
  Update(update::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Sub::Add(cmd) => cmd.call(context).await,
      Sub::Advance(cmd) => cmd.call(context).await,
      Sub::Cancel(cmd) => cmd.call(context).await,
      Sub::Complete(cmd) => cmd.call(context).await,
      Sub::Create(cmd) => cmd.call(context).await,
      Sub::Graph(cmd) => cmd.call(context).await,
      Sub::Link(cmd) => cmd.call(context).await,
      Sub::List(cmd) => cmd.call(context).await,
      Sub::Meta(cmd) => cmd.call(context).await,
      Sub::Next(cmd) => cmd.call(context).await,
      Sub::Remove(cmd) => cmd.call(context).await,
      Sub::Reopen(cmd) => cmd.call(context).await,
      Sub::Show(cmd) => cmd.call(context).await,
      Sub::Status(cmd) => cmd.call(context).await,
      Sub::Tag(cmd) => cmd.call(context).await,
      Sub::Untag(cmd) => cmd.call(context).await,
      Sub::Update(cmd) => cmd.call(context).await,
    }
  }
}
