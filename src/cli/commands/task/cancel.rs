use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{primitives::TaskStatus, task::Patch},
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Cancel a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let before_task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
    let before = serde_json::to_value(&before_task)?;
    let tx = repo::transaction::begin(&conn, project_id, "task cancel").await?;
    let patch = Patch {
      status: Some(TaskStatus::Cancelled),
      ..Default::default()
    };

    let task = repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "tasks", &id.to_string(), "modified", Some(&before)).await?;

    // Cancelled is terminal, so highlight against the all-rows pool.
    let prefix_len = repo::task::shortest_all_prefix(&conn, project_id).await?;

    let short_id = task.id().short();
    self.output.print_entity(&task, &short_id, || {
      SuccessMessage::new("cancelled task")
        .id(task.id().short())
        .prefix_len(prefix_len)
        .field("title", task.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
