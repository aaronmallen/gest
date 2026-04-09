//! CLI entry point and command dispatch for `gest`.

mod commands;
pub mod limit;
pub mod meta_args;
pub mod prompt;
pub mod web_notify;

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
  /// Disable paging of long command output.
  #[get = "pub"]
  #[arg(long, global = true)]
  no_pager: bool,
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

    let tx_before = latest_transaction_id(context).await?;

    let result = command.call(context).await;

    match &result {
      Ok(()) => log::debug!("command succeeded"),
      Err(e) => log::debug!("command failed: {e}"),
    }

    if result.is_ok() {
      let tx_after = latest_transaction_id(context).await?;
      if tx_after != tx_before {
        notify_web_reload_if_possible(context).await;
      }
    }

    result
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
  /// A configuration loading or validation error.
  #[error(transparent)]
  Config(#[from] crate::config::Error),
  /// An editor launch error.
  #[error("{0}")]
  Editor(String),
  /// An I/O error (e.g. writing to the filesystem).
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// A metadata key was not found on the entity.
  #[error("metadata key not found: {0}")]
  MetaKeyNotFound(String),
  /// A serialization error (e.g. snapshotting entity state for transactions).
  #[error(transparent)]
  Serialize(#[from] serde_json::Error),
  /// An error originating in the store layer.
  #[error(transparent)]
  Store(#[from] crate::store::Error),
  /// A TOML serialization error.
  #[error(transparent)]
  TomlSerialize(#[from] toml::ser::Error),
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
  #[command(visible_alias = "p")]
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

/// Notify a running web server (if any) that a mutation occurred so it can refresh
/// browser tabs via SSE. All errors are silently swallowed: server absence is the
/// common case and CLI commands must never pay user-visible latency for this.
async fn notify_web_reload_if_possible(context: &AppContext) {
  let Ok(data_dir) = context.settings().storage().data_dir() else {
    return;
  };

  let _ = web_notify::notify_web_reload(context.gest_dir().as_deref(), &data_dir).await;
}

/// Capture the most recent non-undone transaction id for the current project, if any.
///
/// Used as a watermark for change detection: comparing this value before and after a
/// command runs reveals whether the command resulted in a committed mutation,
/// regardless of which subcommand was invoked. Read-only commands never call
/// `transaction::begin`, so the watermark stays unchanged for them.
async fn latest_transaction_id(context: &AppContext) -> Result<Option<crate::store::model::primitives::Id>, Error> {
  let Some(project_id) = context.project_id() else {
    return Ok(None);
  };

  let conn = context.store().connect().await?;
  let latest = crate::store::repo::transaction::latest_undoable(&conn, project_id).await?;
  Ok(latest.map(|tx| tx.id().clone()))
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
    async fn it_dispatches_to_subcommand() {
      let app = App {
        command: Some(Command::Version(commands::version::Command)),
        no_color: false,
        no_pager: false,
        print_version: false,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        no_pager: false,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      let result = app.call(&context).await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_dispatches_version_flag() {
      let app = App {
        command: None,
        no_color: false,
        no_pager: false,
        print_version: true,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        no_pager: false,
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
        no_pager: false,
        print_version: false,
        verbosity_level: 0,
      };
      let context = AppContext {
        gest_dir: None,
        no_pager: false,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      let result = app.call(&context).await;

      assert!(result.is_ok());
    }
  }

  mod app_context_no_pager {
    use super::*;

    #[tokio::test]
    async fn it_defaults_no_pager_accessor_to_false() {
      let context = AppContext {
        gest_dir: None,
        no_pager: false,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      assert!(!*context.no_pager());
    }

    #[tokio::test]
    async fn it_exposes_no_pager_accessor() {
      let context = AppContext {
        gest_dir: None,
        no_pager: true,
        project_id: None,
        settings: crate::config::Settings::default(),
        store: crate::store::open_temp().await.unwrap().0,
      };

      assert!(*context.no_pager());
    }
  }

  mod app_parse {
    use super::*;

    #[test]
    fn it_defaults_no_pager_to_false() {
      let app = App::try_parse_from(["gest", "version"]).unwrap();

      assert!(!*app.no_pager());
    }

    #[test]
    fn it_parses_no_pager_flag() {
      let app = App::try_parse_from(["gest", "--no-pager", "version"]).unwrap();

      assert!(*app.no_pager());
    }

    #[test]
    fn it_parses_no_pager_flag_after_subcommand() {
      let app = App::try_parse_from(["gest", "version", "--no-pager"]).unwrap();

      assert!(*app.no_pager());
    }
  }
}
