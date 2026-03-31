use std::path::{Path, PathBuf};

use clap::Args;

use crate::{
  cli::{self, AppContext},
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
        .parse::<toml::Value>()
        .map_err(|e| cli::Error::generic(format!("Failed to parse config: {e}")))?
    } else {
      toml::Value::Table(toml::Table::new())
    };

    set_dot_path(&mut toml_value, &self.key, &self.value)?;

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

/// Insert a value into a TOML table at a dot-delimited path, creating intermediate tables as needed.
fn set_dot_path(toml_val: &mut toml::Value, key: &str, value: &str) -> cli::Result<()> {
  let segments: Vec<&str> = key.split('.').collect();

  let mut current = toml_val;
  for segment in &segments[..segments.len() - 1] {
    if !current.is_table() {
      *current = toml::Value::Table(toml::Table::new());
    }
    current = current
      .as_table_mut()
      .unwrap()
      .entry(*segment)
      .or_insert_with(|| toml::Value::Table(toml::Table::new()));
  }

  let last = segments.last().unwrap();
  if let Some(table) = current.as_table_mut() {
    table.insert((*last).to_string(), toml::Value::String(value.to_string()));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod set_dot_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_preserves_sibling_keys() {
      let mut val: toml::Value = toml::from_str("[log]\nlevel = \"warn\"\nother = \"val\"").unwrap();
      set_dot_path(&mut val, "log.level", "debug").unwrap();

      assert_eq!(val["log"]["level"].as_str().unwrap(), "debug");
      assert_eq!(val["log"]["other"].as_str().unwrap(), "val");
    }

    #[test]
    fn it_sets_a_nested_key() {
      let mut val = toml::Value::Table(toml::Table::new());
      set_dot_path(&mut val, "log.level", "debug").unwrap();

      assert_eq!(val["log"]["level"].as_str().unwrap(), "debug");
    }

    #[test]
    fn it_sets_a_top_level_key() {
      let mut val = toml::Value::Table(toml::Table::new());
      set_dot_path(&mut val, "key", "value").unwrap();

      assert_eq!(val["key"].as_str().unwrap(), "value");
    }
  }
}
