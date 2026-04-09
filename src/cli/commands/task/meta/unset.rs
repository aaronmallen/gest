use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::task::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Remove a metadata value from a task at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// The dot-delimited metadata path.
  path: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Remove the metadata value at the given dot-path from the task within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task meta unset: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let task = repo::task::find_required_by_id(&conn, id.clone()).await?;

    let mut metadata = task.metadata().clone();
    if !meta::unset_path(&mut metadata, &self.path) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&task)?;
    let tx = repo::transaction::begin(&conn, project_id, "task meta unset").await?;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "tasks", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::task::find_required_by_id(&conn, id.clone()).await?;
    let short_id = id.short();
    self.output.print_entity(&updated, &short_id, || {
      SuccessMessage::new("unset metadata")
        .id(id.short())
        .field("path", self.path.clone())
        .to_string()
    })?;
    Ok(())
  }
}
