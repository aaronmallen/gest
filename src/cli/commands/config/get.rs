use clap::Args;

use crate::{
  cli::{self, AppContext},
  config::Settings,
};

/// Retrieve a single configuration value by dot-delimited key.
#[derive(Debug, Args)]
pub struct Command {
  /// Dot-delimited config key (e.g. `storage.data_dir`).
  pub key: String,
}

impl Command {
  /// Print the resolved value for the requested key.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let value = resolve_key(&ctx.settings, &self.key)?;
    println!("{value}");
    Ok(())
  }
}

/// Walk the serialized settings tree using dot-separated path segments.
fn resolve_key(settings: &Settings, key: &str) -> cli::Result<String> {
  let json =
    serde_json::to_value(settings).map_err(|e| cli::Error::generic(format!("Failed to serialize config: {e}")))?;
  let mut current = &json;

  for segment in key.split('.') {
    match current.get(segment) {
      Some(v) => current = v,
      None => {
        return Err(cli::Error::generic(format!("Unknown config key: '{key}'")));
      }
    }
  }

  match current {
    serde_json::Value::String(s) => Ok(s.clone()),
    serde_json::Value::Null => Ok("null".to_string()),
    other => Ok(other.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_key {
    use super::*;

    #[test]
    fn it_errors_on_unknown_key() {
      let settings = Settings::default();
      let result = resolve_key(&settings, "nonexistent.key");

      assert!(result.is_err());
    }

    #[test]
    fn it_resolves_top_level_section() {
      let settings = Settings::default();
      let value = resolve_key(&settings, "storage").unwrap();

      assert!(value.contains('{'));
    }
  }
}
