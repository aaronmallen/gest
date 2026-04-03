mod add;
mod advance;
mod cancel;
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

use crate::cli::{self, AppContext};

/// Manage iterations (execution plans grouping tasks into phases).
#[derive(Debug, Args)]
pub struct Command {
  #[command(subcommand)]
  command: IterationCommand,
}

impl Command {
  /// Dispatch to the appropriate iteration subcommand.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    match &self.command {
      IterationCommand::Add(cmd) => cmd.call(ctx),
      IterationCommand::Advance(cmd) => cmd.call(ctx),
      IterationCommand::Cancel(cmd) => cmd.call(ctx),
      IterationCommand::Create(cmd) => cmd.call(ctx),
      IterationCommand::Graph(cmd) => cmd.call(ctx),
      IterationCommand::Link(cmd) => cmd.call(ctx),
      IterationCommand::List(cmd) => cmd.call(ctx),
      IterationCommand::Meta(cmd) => cmd.call(ctx),
      IterationCommand::Next(cmd) => cmd.call(ctx),
      IterationCommand::Remove(cmd) => cmd.call(ctx),
      IterationCommand::Reopen(cmd) => cmd.call(ctx),
      IterationCommand::Show(cmd) => cmd.call(ctx),
      IterationCommand::Status(cmd) => cmd.call(ctx),
      IterationCommand::Tag(cmd) => cmd.call(ctx),
      IterationCommand::Untag(cmd) => cmd.call(ctx),
      IterationCommand::Update(cmd) => cmd.call(ctx),
    }
  }
}

#[derive(Debug, Subcommand)]
enum IterationCommand {
  Add(add::Command),
  Advance(advance::Command),
  Cancel(cancel::Command),
  #[command(visible_alias = "new")]
  Create(create::Command),
  Graph(graph::Command),
  Link(link::Command),
  #[command(visible_alias = "ls")]
  List(list::Command),
  Meta(meta::Command),
  Next(next::Command),
  #[command(visible_alias = "rm")]
  Remove(remove::Command),
  Reopen(reopen::Command),
  #[command(visible_alias = "view")]
  Show(show::Command),
  Status(status::Command),
  Tag(tag::Command),
  Untag(untag::Command),
  #[command(visible_alias = "edit")]
  Update(update::Command),
}
