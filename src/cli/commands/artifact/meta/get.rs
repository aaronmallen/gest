use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::MetaGet, json},
};

/// Get a metadata value from an artifact.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// The metadata key.
  key: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_by_id(&conn, id)
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let value = artifact
      .metadata()
      .get(&self.key)
      .ok_or_else(|| Error::MetaKeyNotFound(self.key.clone()))?;

    self
      .output
      .print_json_or(value, || MetaGet::new(value.to_string()).to_string())?;
    Ok(())
  }
}
