use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::SuccessMessage, json},
};

/// Remove a tag from an artifact.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  /// The tag label to remove.
  label: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact untag: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let before_tag = repo::tag::find_by_label(&conn, &self.label).await?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact untag").await?;
    repo::tag::detach(&conn, EntityType::Artifact, &id, &self.label).await?;
    if let Some(tag) = &before_tag {
      let before = serde_json::to_value(crate::store::model::entity_tag::Model::new(
        EntityType::Artifact,
        id.clone(),
        tag.id().clone(),
      ))?;
      repo::transaction::record_event(
        &conn,
        tx.id(),
        "entity_tags",
        &tag.id().to_string(),
        "deleted",
        Some(&before),
      )
      .await?;
    }

    let artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;
    let prefix_len = if artifact.is_archived() {
      repo::artifact::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::artifact::shortest_active_prefix(&conn, project_id).await?
    };
    let short_id = id.short();
    self.output.print_delete(|| {
      SuccessMessage::new("untagged artifact")
        .id(short_id.clone())
        .prefix_len(prefix_len)
        .field("tag", self.label.clone())
        .to_string()
    })?;
    Ok(())
  }
}
