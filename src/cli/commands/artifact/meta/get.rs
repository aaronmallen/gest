use clap::Args;
use serde_json::{Map, Value};

use crate::{
  AppContext,
  cli::Error,
  store::{meta, repo},
  ui::{components::MetaGet, json},
};

/// Get a metadata value from an artifact by dot-delimited path.
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
  /// Resolve the artifact and print the metadata value at the given dot-path.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact meta get: entry");
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_required_by_id(&conn, id).await?;

    let value =
      meta::resolve_path(artifact.metadata(), &self.path).ok_or_else(|| Error::MetaKeyNotFound(self.path.clone()))?;

    let mut wrapped = Map::new();
    wrapped.insert(self.path.clone(), value.clone());
    let wrapped = Value::Object(wrapped);

    self.output.print_raw_or(
      &wrapped,
      || meta::format_meta_value(value),
      || MetaGet::new(meta::format_meta_value(value)).to_string(),
    )
  }
}
