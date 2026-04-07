use clap::Args;
use toml::Value;

use crate::{AppContext, cli::Error};

/// Get a configuration value by dotted key path.
#[derive(Args, Debug)]
pub struct Command {
  /// The configuration key (e.g. "storage.data_dir", "log.level").
  key: String,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let toml_value = Value::try_from(context.settings()).map_err(std::io::Error::other)?;

    let value = resolve_dotted_key(&toml_value, &self.key);
    match value {
      Some(v) => println!("{v}"),
      None => println!("(not set)"),
    }
    Ok(())
  }
}

fn resolve_dotted_key<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
  let mut current = value;
  for part in key.split('.') {
    current = current.get(part)?;
  }
  Some(current)
}
