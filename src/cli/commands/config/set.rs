use std::path::{Path, PathBuf};

use clap::Args;
use toml::{Table, Value};

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Persist a configuration value to a TOML config file.
#[derive(Debug, Args)]
pub struct Command {
  /// Dot-delimited config key (e.g. `log.level`).
  pub key: String,
  /// Value to assign.
  pub value: String,
  /// Write to the global (user-level) config instead of the project config.
  #[arg(short, long)]
  pub global: bool,
}

/// Whether the write targets the user-wide or project-local config file.
#[derive(Debug)]
enum Scope {
  Global,
  Project,
}

impl Command {
  /// Write the key-value pair to the resolved config file and print confirmation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let scope = if self.global { Scope::Global } else { Scope::Project };

    let config_path = resolve_config_path(&scope)?;

    let mut toml_value = if config_path.exists() {
      let content = std::fs::read_to_string(&config_path)?;
      content
        .parse::<Value>()
        .map_err(|e| cli::Error::generic(format!("Failed to parse config: {e}")))?
    } else {
      Value::Table(Table::new())
    };

    let table = toml_value
      .as_table_mut()
      .ok_or_else(|| cli::Error::generic("Config root is not a TOML table"))?;
    store::meta::set_dot_path(table, &self.key, &self.value)?;

    if let Some(parent) = config_path.parent() {
      std::fs::create_dir_all(parent)?;
    }

    let content =
      toml::to_string_pretty(&toml_value).map_err(|e| cli::Error::generic(format!("Failed to serialize: {e}")))?;
    std::fs::write(&config_path, content)?;

    let scope_label = match scope {
      Scope::Global => "global",
      Scope::Project => "project",
    };

    let msg = format!(
      "Set {}.{} = {} ({})",
      scope_label,
      self.key,
      self.value,
      config_path.display()
    );
    println!("{}", SuccessMessage::new(&msg, &ctx.theme));

    Ok(())
  }
}

/// Return the first path in `names` that exists under `dir`.
fn find_existing(dir: &Path, names: &[&str]) -> Option<PathBuf> {
  for name in names {
    let path = dir.join(name);
    if path.exists() {
      return Some(path);
    }
  }
  None
}

/// Determine which TOML config file to write based on scope and existing files.
fn resolve_config_path(scope: &Scope) -> cli::Result<PathBuf> {
  match scope {
    Scope::Global => {
      if let Some(config_home) = dir_spec::config_home() {
        let config_dir = config_home.join("gest");
        if let Some(existing) = find_existing(&config_dir, &["config.toml"]) {
          return Ok(existing);
        }
        return Ok(config_dir.join("config.toml"));
      }
      Err(cli::Error::generic("Unable to determine global config directory"))
    }
    Scope::Project => {
      let cwd = std::env::current_dir()?;

      if cwd.join(".gest").is_dir() {
        if let Some(existing) = find_existing(&cwd, &[".gest/config.toml"]) {
          return Ok(existing);
        }
        return Ok(cwd.join(".gest/config.toml"));
      }

      if let Some(existing) = find_existing(&cwd, &[".gest.toml"]) {
        return Ok(existing);
      }

      Ok(cwd.join(".gest.toml"))
    }
  }
}
