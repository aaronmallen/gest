use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::artifact::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Set a metadata value on an artifact at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact meta set: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let parsed = if self.as_json {
      serde_json::from_str(&self.value)?
    } else {
      meta::parse_scalar(&self.value)
    };

    let mut metadata = artifact.metadata().clone();
    if !metadata.is_object() {
      metadata = Value::Object(serde_json::Map::new());
    }
    if !meta::set_path(&mut metadata, &self.path, parsed) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&artifact)?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact meta set").await?;

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
        .field("path", self.path.clone())
        .to_string()
    })?;
    Ok(())
  }
}
