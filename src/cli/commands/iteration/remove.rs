use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::SuccessMessage, json},
};

/// Remove a task from an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  iteration: String,
  /// The task ID or prefix.
  task: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let iteration_id = repo::resolve::resolve_id(&conn, "iterations", &self.iteration).await?;
    let task_id = repo::resolve::resolve_id(&conn, "tasks", &self.task).await?;

    let before = serde_json::json!({
      "iteration_id": iteration_id.to_string(),
      "task_id": task_id.to_string(),
    });
    let tx = repo::transaction::begin(&conn, project_id, "iteration remove").await?;
    repo::iteration::remove_task(&conn, &iteration_id, &task_id).await?;
    repo::transaction::record_event(
      &conn,
      tx.id(),
      "iteration_tasks",
      &task_id.to_string(),
      "deleted",
      Some(&before),
    )
    .await?;

    self.output.print_delete(|| {
      SuccessMessage::new("removed task from iteration")
        .field("task", task_id.short())
        .field("iteration", iteration_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
