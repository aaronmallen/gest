use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::SuccessMessage, json},
};

/// Remove a tag from a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// The tag label to remove.
  label: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;

    let before_tag = repo::tag::find_by_label(&conn, &self.label).await?;
    let tx = repo::transaction::begin(&conn, project_id, "task untag").await?;
    repo::tag::detach(&conn, EntityType::Task, &id, &self.label).await?;
    if let Some(tag) = &before_tag {
      let before = serde_json::to_value(crate::store::model::entity_tag::Model::new(
        EntityType::Task,
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

    let short_id = id.short();
    self.output.print_delete(|| {
      SuccessMessage::new("untagged task")
        .id(short_id.clone())
        .field("tag", self.label.clone())
        .to_string()
    })?;
    Ok(())
  }
}
