use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::SuccessMessage, json},
};

/// Add a tag to a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  /// The tag label to add.
  label: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task tag: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;

    let tx = repo::transaction::begin(&conn, project_id, "task tag").await?;
    let tag = repo::tag::attach(&conn, EntityType::Task, &id, &self.label).await?;
    repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;

    // Pool follows the tagged task's status.
    let task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;
    let prefix_len = if task.status().is_terminal() {
      repo::task::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::task::shortest_active_prefix(&conn, project_id).await?
    };

    let short_id = id.short();
    self.output.print_entity(&tag, &short_id, || {
      log::info!("tagged task");
      SuccessMessage::new("tagged task")
        .id(id.short())
        .prefix_len(prefix_len)
        .field("tag", self.label.clone())
        .to_string()
    })?;
    Ok(())
  }
}
