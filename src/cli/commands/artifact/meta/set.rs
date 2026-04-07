use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{model::artifact::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Set a metadata value on an artifact.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// The metadata key.
  key: String,
  /// The metadata value.
  value: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let before = serde_json::to_value(&artifact)?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact meta set").await?;

    let mut metadata = artifact.metadata().clone();
    let value: Value = serde_json::from_str(&self.value).unwrap_or_else(|_| Value::String(self.value.clone()));
    metadata[&self.key] = value;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::artifact::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "artifacts", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;
    let short_id = id.short();
    self.output.print_entity(&updated, &short_id, || {
      SuccessMessage::new("set metadata")
        .id(id.short())
        .field("key", self.key.clone())
        .to_string()
    })?;
    Ok(())
  }
}
