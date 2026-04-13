use clap::Args;

use crate::{AppContext, actions, cli::Error, ui::json};

/// Remove a metadata value from an artifact at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// The dot-delimited metadata path.
  path: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Remove the metadata value at the given dot-path from the artifact within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    actions::meta::unset::<actions::Artifact>(context, &self.id, &self.path, &self.output).await
  }
}
