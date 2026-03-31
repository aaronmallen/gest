//! Application configuration loading and settings.
//!
//! Configuration is loaded hierarchically: global config is merged with
//! per-project TOML files discovered by walking up from the working directory.

pub mod colors;
pub mod env;
mod loader;
mod log;
pub(crate) mod storage;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use self::loader::load;

/// Errors that can occur during configuration loading or resolution.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("invalid config: {0}")]
  Config(String),
  #[error("failed to resolve user's {0} directory")]
  DirectoryNotFound(&'static str),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("{0} is not a directory.")]
  NotADirectory(PathBuf),
}

/// Top-level configuration, composed of color, log, and storage sections.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  colors: colors::Settings,
  log: log::Settings,
  storage: storage::Settings,
}

impl Settings {
  /// Returns the color customization settings.
  pub fn colors(&self) -> &colors::Settings {
    &self.colors
  }

  /// Returns the logging settings.
  pub fn log(&self) -> &log::Settings {
    &self.log
  }

  /// Returns the data storage settings.
  pub fn storage(&self) -> &storage::Settings {
    &self.storage
  }
}
