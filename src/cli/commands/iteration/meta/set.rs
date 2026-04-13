use clap::Args;

use crate::{AppContext, actions, cli::Error, ui::json};

/// Set a metadata value on an iteration at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// The dot-delimited metadata path.
  path: String,
  /// The metadata value (auto-detected scalar unless --as-json is set).
  value: String,
  /// Parse the value as a JSON literal instead of auto-detecting.
  #[arg(long)]
  as_json: bool,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Write the parsed value into the iteration's metadata at the given dot-path within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    actions::meta::set::<actions::Iteration>(context, &self.id, &self.path, &self.value, self.as_json, &self.output)
      .await
  }
}
