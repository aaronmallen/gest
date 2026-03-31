pub mod editor;
pub mod helpers;

mod commands;

use clap::{ArgAction, Parser, Subcommand};
use yansi::hyperlink::HyperlinkExt;

use crate::{config::Settings, ui::theme::Theme};

/// Bundles all runtime context that commands need: resolved settings and theme.
pub(crate) struct AppContext {
  pub(crate) settings: Settings,
  pub(crate) theme: Theme,
}

/// Unified error type for CLI operations, wrapping config, I/O, and store errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Config(#[from] crate::config::Error),
  #[error("{0}")]
  Generic(String),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
  #[error(transparent)]
  Store(#[from] crate::store::Error),
}

impl Error {
  /// Construct a free-form error from any string-like message.
  pub fn generic(msg: impl Into<String>) -> Self {
    Self::Generic(msg.into())
  }
}

/// Convenience alias used throughout the CLI layer.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
#[command(
  about = env!("CARGO_PKG_DESCRIPTION"),
  author = "Aaron Allen <hello@aaronmallen.me>",
  disable_version_flag = true,
  long_about = long_about(),
  name = "gest",
)]
pub(crate) struct Cli {
  #[command(subcommand)]
  command: Option<Command>,
  /// Print version information and exit.
  #[arg(short = 'V', long = "version")]
  print_version: bool,
  /// Increase logging verbosity (repeat for more detail).
  #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
  verbose: u8,
}

impl Cli {
  fn call(&self, mut settings: Settings) -> Result<()> {
    if self.print_version {
      let theme = Theme::from_config(&settings);
      let ctx = AppContext {
        settings,
        theme,
      };
      return commands::version::Command.call(&ctx);
    }

    let Some(command) = &self.command else {
      use clap::CommandFactory;
      Self::command().print_long_help()?;
      return Ok(());
    };

    let env_level = crate::config::env::GEST_LOG_LEVEL.value().ok();
    let level = crate::logger::resolve_level(self.verbose, env_level.as_deref(), settings.log().level());
    let theme = Theme::from_config(&settings);
    crate::logger::init(level, &theme);

    let cwd = std::env::current_dir()?;
    settings.resolve_storage(cwd)?;

    log::debug!("log level set to {level}");
    log::debug!("data directory: {}", settings.data_dir().display());

    let ctx = AppContext {
      settings,
      theme,
    };
    command.call(&ctx)
  }
}

#[derive(Debug, Subcommand)]
enum Command {
  Artifact(commands::artifact::Command),
  Config(commands::config::Command),
  Generate(commands::generate::Command),
  Init(commands::init::Command),
  Iteration(commands::iteration::Command),
  Search(commands::search::Command),
  SelfUpdate(commands::self_update::Command),
  Serve(commands::serve::Command),
  Task(commands::task::Command),
  Version(commands::version::Command),
}

impl Command {
  fn call(&self, ctx: &AppContext) -> Result<()> {
    match self {
      Self::Artifact(cmd) => cmd.call(ctx),
      Self::Config(cmd) => cmd.call(ctx),
      Self::Generate(cmd) => cmd.call(ctx),
      Self::Init(cmd) => cmd.call(ctx),
      Self::Iteration(cmd) => cmd.call(ctx),
      Self::Search(cmd) => cmd.call(ctx),
      Self::SelfUpdate(cmd) => cmd.call(ctx),
      Self::Serve(cmd) => cmd.call(ctx),
      Self::Task(cmd) => cmd.call(ctx),
      Self::Version(cmd) => cmd.call(ctx),
    }
  }
}

/// Entry point for the CLI: loads configuration then parses and dispatches the command.
pub fn run() -> Result<()> {
  let verbosity = pre_parse_verbosity();
  let early_level = crate::logger::resolve_level(verbosity, None, None);
  crate::logger::init_early(early_level);

  let cwd = std::env::current_dir()?;
  let settings = crate::config::load(&cwd)?;
  Cli::parse().call(settings)
}

/// Count `-v` / `--verbose` occurrences in an argument iterator, stopping at `--`.
fn count_verbosity_flags(args: impl Iterator<Item = String>) -> u8 {
  let mut count: u8 = 0;
  for arg in args {
    if arg == "--verbose" {
      count = count.saturating_add(1);
    } else if arg == "--" {
      break;
    } else if arg.starts_with('-') && !arg.starts_with("--") {
      count = count.saturating_add(arg.chars().filter(|&c| c == 'v').count() as u8);
    }
  }
  count
}

/// Build the `--help` long-about string including a styled banner.
fn long_about() -> String {
  let theme = Theme::default();
  let banner = crate::ui::composites::banner::Banner::new(
    env!("CARGO_PKG_VERSION"),
    std::env::consts::OS,
    "",
    "",
    "aaronmallen",
    &theme,
  )
  .hide_version();
  let url = "https://gest.aaronmallen.dev";
  let link = url.link(url).fg(crate::ui::colors::AZURE).underline();
  format!(
    "\n{banner}\n\n\
     {desc}\n\n\
     {link}\n\n\
     Gest provides a lightweight, file-based system for organizing the artifacts, specs, ADRs, \
     and task backlogs that AI coding agents produce. Instead of letting generated context scatter \
     across chat logs and throwaway files, gest stores it in a structured, version-controlled \
     directory right inside your repo — so every decision, plan, and backlog item travels with the \
     code it describes. It includes a local web dashboard for browsing and managing your project's \
     knowledge base, and a CLI that integrates naturally into agent-driven workflows.",
    desc = env!("CARGO_PKG_DESCRIPTION"),
  )
}

/// Quick pre-parse of verbosity from `std::env::args` so logging is active before full clap parse.
fn pre_parse_verbosity() -> u8 {
  count_verbosity_flags(std::env::args().skip(1))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod count_verbosity_flags {
    use pretty_assertions::assert_eq;

    use super::*;

    fn flags(args: &[&str]) -> u8 {
      count_verbosity_flags(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn it_counts_clustered_v() {
      assert_eq!(flags(&["-vvv"]), 3);
    }

    #[test]
    fn it_counts_long_verbose() {
      assert_eq!(flags(&["--verbose", "--verbose"]), 2);
    }

    #[test]
    fn it_counts_mixed_short_and_long() {
      assert_eq!(flags(&["-vv", "--verbose"]), 3);
    }

    #[test]
    fn it_counts_single_v() {
      assert_eq!(flags(&["-v"]), 1);
    }

    #[test]
    fn it_ignores_long_flags_containing_v() {
      assert_eq!(flags(&["--version"]), 0);
    }

    #[test]
    fn it_returns_zero_with_no_flags() {
      assert_eq!(flags(&["task", "show", "abc"]), 0);
    }

    #[test]
    fn it_stops_at_double_dash() {
      assert_eq!(flags(&["-v", "--", "-vv"]), 1);
    }
  }
}
