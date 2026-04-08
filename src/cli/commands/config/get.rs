use clap::Args;
use toml::Value;

use crate::{AppContext, cli::Error, ui::json};

/// Get a configuration value by dotted key path.
#[derive(Args, Debug)]
pub struct Command {
  /// The configuration key (e.g. "storage.data_dir", "log.level").
  key: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("config get: entry");
    let toml_value = Value::try_from(context.settings()).map_err(std::io::Error::other)?;
    let value = resolve_dotted_key(&toml_value, &self.key).cloned();

    let human = value
      .as_ref()
      .map_or_else(|| "(not set)".to_string(), display_toml_value);
    self.output.print_entity(&value, &human, || human.clone())?;
    Ok(())
  }
}

fn display_toml_value(value: &Value) -> String {
  match value {
    Value::String(s) => s.clone(),
    other => other.to_string(),
  }
}

fn resolve_dotted_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
  let mut current = value;
  for part in key.split('.') {
    current = current.get(part)?;
  }
  Some(current)
}
