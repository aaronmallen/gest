//! Typed environment variable definitions used across the application.

use std::path::PathBuf;

use typed_env::{Envar, EnvarDef};

/// The user's preferred text editor (`$EDITOR`).
pub static EDITOR: Envar<String> = Envar::on_demand("EDITOR", || EnvarDef::Unset);

/// Override path for the global configuration file.
pub static GEST_CONFIG: Envar<PathBuf> = Envar::on_demand("GEST_CONFIG", || EnvarDef::Unset);

/// Override path for the artifact storage directory.
pub static GEST_ARTIFACT_DIR: Envar<PathBuf> = Envar::on_demand("GEST_ARTIFACT_DIR", || EnvarDef::Unset);

/// Override path for the data storage directory.
pub static GEST_DATA_DIR: Envar<PathBuf> = Envar::on_demand("GEST_DATA_DIR", || EnvarDef::Unset);

/// Override path for the iteration storage directory.
pub static GEST_ITERATION_DIR: Envar<PathBuf> = Envar::on_demand("GEST_ITERATION_DIR", || EnvarDef::Unset);

/// Override for the log level filter (e.g. `"debug"`, `"trace"`).
pub static GEST_LOG_LEVEL: Envar<String> = Envar::on_demand("GEST_LOG_LEVEL", || EnvarDef::Unset);

/// Override path for the task storage directory.
pub static GEST_TASK_DIR: Envar<PathBuf> = Envar::on_demand("GEST_TASK_DIR", || EnvarDef::Unset);

/// The user's preferred visual editor (`$VISUAL`), checked before `$EDITOR`.
pub static VISUAL: Envar<String> = Envar::on_demand("VISUAL", || EnvarDef::Unset);
