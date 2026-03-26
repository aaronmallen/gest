pub mod editor;
pub mod helpers;

mod commands;

use clap::{ArgAction, Parser, Subcommand};

use crate::{
  Result,
  config::Config,
  ui::{components::Banner, theme::Theme},
};

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
  /// Print version information
  #[arg(short = 'V', long = "version")]
  print_version: bool,
  /// Increase log verbosity (repeat for more detail, e.g. -vv)
  #[arg(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
  verbose: u8,
}

impl Cli {
  fn call(&self, config: &Config) -> Result<()> {
    if self.print_version {
      return commands::version::Command.call();
    }

    let Some(command) = &self.command else {
      use clap::CommandFactory;
      Self::command().print_long_help()?;
      return Ok(());
    };

    // Resolve the final level using the full precedence chain (CLI > env > config > default)
    // and apply theme styles.  The logger was already registered by `init_early` in `run()`.
    let env_level = crate::config::env::GEST_LOG_LEVEL.value().ok();
    let level = crate::logger::resolve_level(self.verbose, env_level.as_deref(), config.log.level.as_deref());
    let theme = Theme::from_config(config);
    crate::logger::init(level, &theme);

    log::debug!("log level set to {level}");
    log::debug!("data directory: {}", crate::config::data_dir(config)?.display());
    command.call(config, &theme)
  }
}

#[derive(Debug, Subcommand)]
enum Command {
  Artifact(commands::artifact::Command),
  Config(commands::config::Command),
  Generate(commands::generate::Command),
  Init(commands::init::Command),
  Search(commands::search::Command),
  SelfUpdate(commands::self_update::Command),
  Task(commands::task::Command),
  Version(commands::version::Command),
}

impl Command {
  fn call(&self, config: &Config, theme: &Theme) -> Result<()> {
    match self {
      Self::Artifact(cmd) => cmd.call(config, theme),
      Self::Config(cmd) => cmd.call(config, theme),
      Self::Generate(cmd) => cmd.call(),
      Self::Init(cmd) => cmd.call(config, theme),
      Self::Search(cmd) => cmd.call(config, theme),
      Self::SelfUpdate(cmd) => cmd.call(config, theme),
      Self::Task(cmd) => cmd.call(config, theme),
      Self::Version(cmd) => cmd.call(),
    }
  }
}

pub fn run() -> Result<()> {
  // Pre-parse verbosity so the logger is active during config discovery.
  let verbosity = pre_parse_verbosity();
  let early_level = crate::logger::resolve_level(verbosity, None, None);
  crate::logger::init_early(early_level);

  let config = crate::config::load()?;
  Cli::parse().call(&config)
}

/// Count `-v` / `--verbose` occurrences in an argument iterator.
fn count_verbosity_flags(args: impl Iterator<Item = String>) -> u8 {
  let mut count: u8 = 0;
  for arg in args {
    if arg == "--verbose" {
      count = count.saturating_add(1);
    } else if arg == "--" {
      break;
    } else if arg.starts_with('-') && !arg.starts_with("--") {
      // Short flag cluster, e.g. `-vv` or `-v`
      count = count.saturating_add(arg.chars().filter(|&c| c == 'v').count() as u8);
    }
  }
  count
}

fn long_about() -> String {
  format!(
    "\n{}\n\n{}",
    Banner::new().with_color().with_author(),
    env!("CARGO_PKG_DESCRIPTION"),
  )
}

/// Scan `std::env::args()` for `-v` / `--verbose` flags and return the count.
///
/// This intentionally avoids a full clap parse so it can run before anything
/// else.  It handles `-v`, `-vv`, `-vvv`, combined short flags like `-vvv`,
/// and `--verbose` (each occurrence counts as 1).
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
    fn it_counts_single_v() {
      assert_eq!(flags(&["-v"]), 1);
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
    fn it_returns_zero_with_no_flags() {
      assert_eq!(flags(&["task", "show", "abc"]), 0);
    }

    #[test]
    fn it_stops_at_double_dash() {
      assert_eq!(flags(&["-v", "--", "-vv"]), 1);
    }

    #[test]
    fn it_ignores_long_flags_containing_v() {
      assert_eq!(flags(&["--version"]), 0);
    }
  }
}
