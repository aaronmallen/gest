use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::SuccessMessage, json},
};

/// Add a task to an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  iteration: String,
  /// The task ID or prefix.
  task: String,
  /// The phase to add the task to (default: 1).
  #[arg(long, short, default_value = "1")]
  phase: u32,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration add: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let iteration_id = repo::resolve::resolve_id(&conn, "iterations", &self.iteration).await?;
    let task_id = repo::resolve::resolve_id(&conn, "tasks", &self.task).await?;

    let tx = repo::transaction::begin(&conn, project_id, "iteration add").await?;
    repo::iteration::add_task(&conn, &iteration_id, &task_id, self.phase).await?;
    repo::transaction::record_event(&conn, tx.id(), "iteration_tasks", &task_id.to_string(), "created", None).await?;

    let short_id = task_id.short();
    let result = serde_json::json!({
      "task_id": task_id.to_string(),
      "iteration_id": iteration_id.to_string(),
      "phase": self.phase,
    });
    self.output.print_entity(&result, &short_id, || {
      log::info!("added task to iteration");
      SuccessMessage::new("added task to iteration")
        .field("task", task_id.short())
        .field("iteration", iteration_id.short())
        .field("phase", self.phase.to_string())
        .to_string()
    })?;
    Ok(())
  }
}
