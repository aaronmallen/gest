use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::task::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Set a metadata value on a task at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
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
  /// Write the parsed value into the task's metadata at the given dot-path within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task meta set: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_required_by_id(&conn, id.clone()).await?;

    let parsed = if self.as_json {
      serde_json::from_str(&self.value)?
    } else {
      meta::parse_scalar(&self.value)
    };

    let mut metadata = task.metadata().clone();
    if !metadata.is_object() {
      metadata = Value::Object(serde_json::Map::new());
    }
    if !meta::set_path(&mut metadata, &self.path, parsed) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&task)?;
    let tx = repo::transaction::begin(&conn, project_id, "task meta set").await?;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "tasks", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::task::find_required_by_id(&conn, id.clone()).await?;
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
