use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{model::iteration::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Set a metadata value on an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
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
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let before = serde_json::to_value(&iteration)?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration meta set").await?;

    let mut metadata = iteration.metadata().clone();
    let value: Value = serde_json::from_str(&self.value).unwrap_or_else(|_| Value::String(self.value.clone()));
    metadata[&self.key] = value;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::iteration::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "iterations", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::iteration::find_by_id(&conn, id.clone())
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
