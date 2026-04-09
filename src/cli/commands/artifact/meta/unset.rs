use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::artifact::Patch, repo},
  ui::{components::SuccessMessage, json},
};

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
    log::debug!("artifact meta unset: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_required_by_id(&conn, id.clone()).await?;

    let mut metadata = artifact.metadata().clone();
    if !meta::unset_path(&mut metadata, &self.path) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&artifact)?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact meta unset").await?;

    let patch = Patch {
      metadata: Some(metadata),
      ..Default::default()
    };
    repo::artifact::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "artifacts", &id.to_string(), "modified", Some(&before)).await?;

    let updated = repo::artifact::find_required_by_id(&conn, id.clone()).await?;
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
