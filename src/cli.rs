//! CLI entry point and command dispatch for `gest`.

mod commands;
pub mod meta_args;

use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use getset::Getters;
use yansi::Paint;

use crate::{
  AppContext,
  ui::{components::Banner, style},
};

/// Root CLI definition parsed by `clap`.
///
/// Metadata (description, author, version) is pulled from `Cargo.toml` at
/// compile time so there is a single source of truth.
#[derive(Debug, Getters, Parser)]
#[command(
  about = env!("CARGO_PKG_DESCRIPTION"),
  author = "Aaron Allen <hello@aaronmallen.me>",
  disable_version_flag = true,
  long_about = long_about(),
  name = "gest",
)]
pub struct App {
  /// The subcommand to execute.
  #[command(subcommand)]
  command: Option<Command>,
  /// Disable ANSI color output.
  #[arg(long, global = true)]
  no_color: bool,
  /// Print the current version, platform info, and check for available updates.
  #[arg(long = "version", short = 'V')]
  print_version: bool,
  /// Increase log verbosity (`-v` = info, `-vv` = debug, `-vvv` = trace).
  #[get = "pub"]
  #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
  verbosity_level: u8,
}

impl App {
  /// Dispatch to the selected subcommand.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    if self.no_color {
      yansi::disable();
    }

    if self.print_version {
      return commands::version::Command.call(context).await;
    }

    let Some(command) = &self.command else {
      Self::command().print_long_help()?;
      return Ok(());
    };

    if command.requires_project() && context.project_id().is_none() {
      return Err(Error::UninitializedProject);
    }

    command.call(context).await
  }
}

/// Top-level error type for the CLI layer.
///
/// Variants are added as new subsystems (config, storage, etc.) are wired in.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// An invalid command-line argument.
  #[error("{0}")]
  Argument(String),
  /// An artifact repository error.
  #[error(transparent)]
  ArtifactRepo(#[from] crate::store::repo::artifact::Error),
  /// An author repository error.
  #[error(transparent)]
  AuthorRepo(#[from] crate::store::repo::author::Error),
  /// A configuration loading or validation error.
  #[error(transparent)]
  Config(#[from] crate::config::Error),
  /// An editor launch error.
  #[error("{0}")]
  Editor(String),
  /// An I/O error (e.g. writing to the filesystem).
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// An iteration repository error.
  #[error(transparent)]
  IterationRepo(#[from] crate::store::repo::iteration::Error),
  /// A metadata key was not found on the entity.
  #[error("metadata key not found: {0}")]
  MetaKeyNotFound(String),
  /// A note repository error.
  #[error(transparent)]
  NoteRepo(#[from] crate::store::repo::note::Error),
  /// A project repository error.
  #[error(transparent)]
  ProjectRepo(#[from] crate::store::repo::project::Error),
  /// A relationship repository error.
  #[error(transparent)]
  RelationshipRepo(#[from] crate::store::repo::relationship::Error),
  /// An ID prefix resolution error.
  #[error(transparent)]
  Resolve(#[from] crate::store::repo::resolve::Error),
  /// A search error.
  #[error(transparent)]
  SearchRepo(#[from] crate::store::repo::search::Error),
  /// A serialization error (e.g. snapshotting entity state for transactions).
  #[error(transparent)]
  Serialize(#[from] serde_json::Error),
  /// An error opening or connecting to the store.
  #[error(transparent)]
  Store(#[from] crate::store::Error),
  /// A tag repository error.
  #[error(transparent)]
  TagRepo(#[from] crate::store::repo::tag::Error),
  /// A task repository error.
  #[error(transparent)]
  TaskRepo(#[from] crate::store::repo::task::Error),
  /// A transaction repository error.
  #[error(transparent)]
  TransactionRepo(#[from] crate::store::repo::transaction::Error),
  /// The current directory has not been initialized as a gest project.
  #[error("not a gest project (run `gest init` to initialize)")]
  UninitializedProject,
}

/// Enum of all available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
  /// Manage artifacts.
  #[command(alias = "a")]
  Artifact(commands::artifact::Command),
  /// View or modify configuration.
  Config(commands::config::Command),
  /// Generate shell completions and man pages.
  Generate(commands::generate::Command),
  /// Initialize gest for the current directory.
  Init(commands::init::Command),
  /// Manage iterations.
  #[command(alias = "i")]
  Iteration(commands::iteration::Command),
  /// Import v0.4.x flat-file data into the current project store.
  Migrate(commands::migrate::Command),
  /// Show or manage the current project.
  Project(commands::project::Command),
  /// Search across all entity types.
  #[command(alias = "grep")]
  Search(commands::search::Command),
  /// Download and install the latest release from GitHub.
  #[command(name = "self-update")]
  SelfUpdate(commands::self_update::Command),
  /// Start the web dashboard server.
  #[command(alias = "s")]
  Serve(commands::serve::Command),
  /// List all tags.
  Tag(commands::tag::Command),
  /// Manage tasks.
  #[command(alias = "t")]
  Task(commands::task::Command),
  /// Undo the last command.
  #[command(alias = "u")]
  Undo(commands::undo::Command),
  /// Print the current version, platform info, and check for available updates.
  Version(commands::version::Command),
}

impl Command {
  /// Dispatch to the matched subcommand's handler.
  async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match self {
      Self::Artifact(command) => command.call(context).await,
      Self::Config(command) => command.call(context).await,
      Self::Generate(command) => command.call(context).await,
      Self::Init(command) => command.call(context).await,
      Self::Iteration(command) => command.call(context).await,
      Self::Migrate(command) => command.call(context).await,
      Self::Project(command) => command.call(context).await,
      Self::Search(command) => command.call(context).await,
      Self::SelfUpdate(command) => command.call(context).await,
      Self::Serve(command) => command.call(context).await,
      Self::Tag(command) => command.call(context).await,
      Self::Task(command) => command.call(context).await,
      Self::Undo(command) => command.call(context).await,
      Self::Version(command) => command.call(context).await,
    }
  }

  /// Whether this subcommand requires an initialized project.
  fn requires_project(&self) -> bool {
    match self {
      Self::Config(_)
      | Self::Generate(_)
      | Self::Init(_)
      | Self::Migrate(_)
      | Self::Project(_)
      | Self::SelfUpdate(_)
      | Self::Undo(_)
      | Self::Version(_) => false,
      Self::Tag(cmd) => cmd.requires_project(),
      Self::Artifact(_) | Self::Iteration(_) | Self::Search(_) | Self::Serve(_) | Self::Task(_) => true,
    }
  }
}

/// Build the `--help` long description: banner, one-liner, docs link, and extended blurb.
fn long_about() -> String {
  let theme = style::global();
  let banner = Banner::new().with_author();
  let description = env!("CARGO_PKG_DESCRIPTION");
  let doc_site_url = "https://gest.aaronmallen.dev";
  let painted_doc_site = doc_site_url.paint(*theme.markdown_link());
  let doc_site_link = painted_doc_site.link(doc_site_url);
  format!(
    "\n{banner}\n\n{description}\n\n{doc_site_link}\n\n\
    Gest provides a lightweight, file-based system for organizing the artifacts, specs, ADRs, \
    and task backlogs that AI coding agents produce. Instead of letting generated context scatter \
    across chat logs and throwaway files, gest stores it in a structured, version-controlled \
    directory right inside your repo — so every decision, plan, and backlog item travels with the \
    code it describes. It includes a local web dashboard for browsing and managing your project's \
    knowledge base, and a CLI that integrates naturally into agent-driven workflows."
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  mod app_call {
    use super::*;

    #[tokio::test]
    async fn it_dispatches_version_flag() {
      let app = App {
        command: None,
        no_color: false,
        print_version: true,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      let result = app.call(&context).await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_prints_long_help_when_no_command() {
      let app = App {
        command: None,
        no_color: false,
        print_version: false,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      let result = app.call(&context).await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_dispatches_to_subcommand() {
      let app = App {
        command: Some(Command::Version(commands::version::Command)),
        no_color: false,
        print_version: false,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      let result = app.call(&context).await;

      assert!(result.is_ok());
    }
  }
}
