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

fn long_about() -> String {
  format!(
    "\n{}\n\n{}",
    Banner::new().with_color().with_author(),
    env!("CARGO_PKG_DESCRIPTION"),
  )
}

pub fn run() -> Result<()> {
  let config = crate::config::load()?;
  Cli::parse().call(&config)
}
