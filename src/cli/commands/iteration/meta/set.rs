use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::iteration::Patch, repo},
  ui::{components::SuccessMessage, json},
};

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
    log::debug!("iteration meta set: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    let parsed = if self.as_json {
      serde_json::from_str(&self.value)?
    } else {
      meta::parse_scalar(&self.value)
    };

    let mut metadata = iteration.metadata().clone();
    if !metadata.is_object() {
      metadata = Value::Object(serde_json::Map::new());
    }
    if !meta::set_path(&mut metadata, &self.path, parsed) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&iteration)?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration meta set").await?;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::iteration::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "iterations", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::iteration::find_required_by_id(&conn, id.clone()).await?;
    let short_id = id.short();
    self.output.print_entity(&updated, &short_id, || {
      SuccessMessage::new("set metadata")
        .id(id.short())
        .field("path", self.path.clone())
        .to_string()
    })?;
    Ok(())
  }
}
