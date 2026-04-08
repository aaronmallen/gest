use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::SuccessMessage, json},
};

/// Add a tag to an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// The tag label to add.
  label: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration tag: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration tag").await?;
    let tag = repo::tag::attach(&conn, EntityType::Iteration, &id, &self.label).await?;
    repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;

    let prefix_len = repo::iteration::shortest_active_prefix(&conn, project_id).await?;

    let short_id = id.short();
    self.output.print_entity(&tag, &short_id, || {
      SuccessMessage::new("tagged iteration")
        .id(id.short())
        .prefix_len(prefix_len)
        .field("tag", self.label.clone())
        .to_string()
    })?;
    Ok(())
  }
}
