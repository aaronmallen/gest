mod add;
mod create;
mod graph;
mod link;
mod list;
mod meta;
mod remove;
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

#[derive(Debug, Subcommand)]
enum IterationCommand {
  Add(add::Command),
  Create(create::Command),
  Graph(graph::Command),
  Link(link::Command),
  List(list::Command),
  Meta(meta::Command),
  Remove(remove::Command),
  Show(show::Command),
  Status(status::Command),
  Tag(tag::Command),
  Untag(untag::Command),
  Update(update::Command),
}

impl Command {
  /// Dispatch to the appropriate iteration subcommand.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    match &self.command {
      IterationCommand::Add(cmd) => cmd.call(ctx),
      IterationCommand::Create(cmd) => cmd.call(ctx),
      IterationCommand::Graph(cmd) => cmd.call(ctx),
      IterationCommand::Link(cmd) => cmd.call(ctx),
      IterationCommand::List(cmd) => cmd.call(ctx),
      IterationCommand::Meta(cmd) => cmd.call(ctx),
      IterationCommand::Remove(cmd) => cmd.call(ctx),
      IterationCommand::Show(cmd) => cmd.call(ctx),
      IterationCommand::Status(cmd) => cmd.call(ctx),
      IterationCommand::Tag(cmd) => cmd.call(ctx),
      IterationCommand::Untag(cmd) => cmd.call(ctx),
      IterationCommand::Update(cmd) => cmd.call(ctx),
    }
  }
}
