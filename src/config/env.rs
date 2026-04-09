use std::path::PathBuf;

use typed_env::{Envar, EnvarDef};

use crate::logging::LevelFilter;

/// Path to the global gest configuration file. When set, this overrides the default XDG config location
/// (`$XDG_CONFIG_HOME/gest/config.toml`).
pub static GEST_CONFIG: Envar<PathBuf> = Envar::on_demand("GEST_CONFIG", || EnvarDef::Unset);

/// Log level override. Accepts level names (`debug`, `info`, …) or numeric values (`0`–`5`).
/// When set, this takes precedence over the `[log].level` config-file value.
pub static GEST_LOG__LEVEL: Envar<LevelFilter> = Envar::on_demand("GEST_LOG__LEVEL", || EnvarDef::Unset);

/// Path to the gest cache directory. When set, this overrides the default XDG cache location
/// (`$XDG_CACHE_HOME/gest`). Used for ephemeral, regeneratable artifacts such as the avatar cache.
pub static GEST_STORAGE__CACHE_DIR: Envar<PathBuf> = Envar::on_demand("GEST_STORAGE__CACHE_DIR", || EnvarDef::Unset);

/// Path to the gest data directory. When set, this overrides the default XDG data location (`$XDG_DATA_HOME/gest`).
pub static GEST_STORAGE__DATA_DIR: Envar<PathBuf> = Envar::on_demand("GEST_STORAGE__DATA_DIR", || EnvarDef::Unset);

/// Enable or disable local file sync. Set to `false` to skip import/export of `.gest/` files.
/// Defaults to enabled when unset.
pub static GEST_STORAGE__SYNC: Envar<bool> = Envar::on_demand("GEST_STORAGE__SYNC", || EnvarDef::Unset);
