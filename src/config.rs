//! Layered configuration loading and typed settings for the gest CLI.
//!
//! Settings are merged from a global config file, per-directory config files
//! walked from filesystem root down to `$CWD`, and `GEST_`-prefixed environment
//! variables (highest precedence).

pub mod colors;
/// Database connection settings (URL or individual components).
mod database;
/// Environment variable definitions used by the config system.
pub mod env;
mod loader;
/// Log-related configuration (level filtering).
mod log;
/// Pager-related configuration (`[pager]` table).
mod pager;
/// Web server configuration (`[serve]` table).
mod serve;
/// Storage-related configuration (data directory resolution).
mod storage;

use std::io::Error as IoError;

use getset::Getters;
pub use loader::{active_global_config_path, active_project_config_paths, load};
use serde::{Deserialize, Serialize};
use toml::de::Error as TomlDeError;

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// A config file contained invalid TOML.
  #[error(transparent)]
  Parse(#[from] TomlDeError),
  /// An I/O error occurred while reading a config file.
  #[error(transparent)]
  Read(#[from] IoError),
  /// The XDG base directory could not be resolved (e.g. `$HOME` is unset).
  #[error("could not resolve user's {0} directory")]
  XDGDirNotFound(&'static str),
}

/// Gest configuration settings, merged from global and per-directory config files.
#[derive(Clone, Debug, Default, Deserialize, Getters, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  /// Settings for color and style customization (`[colors]` table).
  #[get = "pub"]
  colors: colors::Settings,
  /// Settings for database connectivity (`[database]` table).
  #[get = "pub"]
  database: database::Settings,
  /// Settings for log output (`[log]` table).
  #[get = "pub"]
  log: log::Settings,
  /// Settings for the pager helper (`[pager]` table).
  #[get = "pub"]
  pager: pager::Settings,
  /// Settings for the built-in web dashboard (`[serve]` table).
  #[get = "pub"]
  serve: serve::Settings,
  /// Settings for data storage locations (`[storage]` table).
  #[get = "pub"]
  storage: storage::Settings,
}
