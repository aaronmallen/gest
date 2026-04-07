use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{meta, model::iteration::Patch, repo},
  ui::{components::SuccessMessage, json},
};

/// Remove a metadata value from an iteration at a dot-delimited path.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// The dot-delimited metadata path.
  path: String,
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

    let mut metadata = iteration.metadata().clone();
    if !meta::unset_path(&mut metadata, &self.path) {
      return Err(Error::MetaKeyNotFound(self.path.clone()));
    }

    let before = serde_json::to_value(&iteration)?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration meta unset").await?;

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
      SuccessMessage::new("unset metadata")
        .id(id.short())
        .field("path", self.path.clone())
        .to_string()
    })?;
    Ok(())
  }
}
