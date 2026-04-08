//! The `migrate` subcommand — imports legacy data into the current project store.

pub(crate) mod discover;
mod v0_4;

use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::{AppContext, cli::Error};

/// Source format version to migrate from.
#[derive(Clone, Debug, ValueEnum)]
enum Version {
  /// v0.4.x flat-file `.gest/` directory format.
  #[value(name = "v0.4")]
  V0_4,
}

/// Import data from a previous gest version into the current project store.
#[derive(Args, Debug)]
pub struct Command {
  /// The source format version to migrate from.
  #[arg(long)]
  from: Version,

  /// Path to the legacy data directory (defaults to auto-discovery).
  #[arg(long)]
  path: Option<PathBuf>,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("migrate: entry");
    let source = match &self.path {
      Some(p) => p.clone(),
      None => {
        let cwd = std::env::current_dir()?;
        discover::find_legacy_dir(&cwd)?
      }
    };

    match self.from {
      Version::V0_4 => v0_4::run(context, &source).await,
    }
  }
}
