//! Application configuration loading and settings.
//!
//! Configuration is loaded hierarchically: global config is merged with
//! per-project TOML files discovered by walking up from the working directory.

pub mod colors;
pub mod env;
mod loader;
mod log;
pub(crate) mod storage;

use std::path::{Path, PathBuf};

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
  #[serde(skip)]
  resolved_artifact_dir: PathBuf,
  #[serde(skip)]
  resolved_data_dir: PathBuf,
  #[serde(skip)]
  resolved_iteration_dir: PathBuf,
  #[serde(skip)]
  resolved_task_dir: PathBuf,
}

impl Settings {
  /// The resolved artifact storage directory.
  pub fn artifact_dir(&self) -> &Path {
    &self.resolved_artifact_dir
  }

  /// Returns the color customization settings.
  pub fn colors(&self) -> &colors::Settings {
    &self.colors
  }

  /// The resolved base data directory.
  pub fn data_dir(&self) -> &Path {
    &self.resolved_data_dir
  }

  /// The resolved iteration storage directory.
  pub fn iteration_dir(&self) -> &Path {
    &self.resolved_iteration_dir
  }

  /// Returns the logging settings.
  pub fn log(&self) -> &log::Settings {
    &self.log
  }

  /// Resolve storage paths from the working directory.
  ///
  /// Must be called before accessing `data_dir`, `artifact_dir`, `task_dir`,
  /// or `iteration_dir`. Resolves the base data directory and all per-entity
  /// directories according to env var / config / fallback precedence.
  pub fn resolve_storage(&mut self, cwd: PathBuf) -> Result<(), Error> {
    let data_dir = self.storage.resolve_data_dir(cwd)?;
    self.resolve_storage_at(data_dir);
    Ok(())
  }

  /// Set storage paths for an already-known data directory, skipping discovery.
  pub fn resolve_storage_at(&mut self, data_dir: PathBuf) {
    self.resolved_data_dir = data_dir;
    self.resolved_artifact_dir = self.storage.resolve_artifact_dir(&self.resolved_data_dir);
    self.resolved_iteration_dir = self.storage.resolve_iteration_dir(&self.resolved_data_dir);
    self.resolved_task_dir = self.storage.resolve_task_dir(&self.resolved_data_dir);
  }

  /// Returns the data storage settings.
  pub fn storage(&self) -> &storage::Settings {
    &self.storage
  }

  /// The resolved task storage directory.
  pub fn task_dir(&self) -> &Path {
    &self.resolved_task_dir
  }
}
