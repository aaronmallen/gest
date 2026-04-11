use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{
      primitives::{EntityType, TaskStatus},
      task::Patch,
    },
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Mark a task as done.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Transition the resolved task to `done` within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task complete: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
    let before_task = repo::task::find_by_id(&conn, id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;
    let before = serde_json::to_value(&before_task)?;
    let tx = repo::transaction::begin(&conn, project_id, "task complete").await?;
    let patch = Patch {
      status: Some(TaskStatus::Done),
      ..Default::default()
    };

    let task = repo::task::update(&conn, &id, &patch).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "tasks",
      &id.to_string(),
      "modified",
      Some(&before),
      Some("completed"),
      Some(&before_task.status().to_string()),
      Some(&task.status().to_string()),
    )
    .await?;

    let envelope = Envelope::load_one(&conn, EntityType::Task, task.id(), &task, true).await?;

    let prefix_map = repo::task::per_id_prefix_lengths(&conn, project_id).await?;
    let prefix_len = prefix_map.get(&id.to_string()).copied().unwrap_or(1);

    let short_id = task.id().short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("completed task");
      SuccessMessage::new("completed task")
        .id(task.id().short())
        .prefix_len(prefix_len)
        .field("title", task.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
