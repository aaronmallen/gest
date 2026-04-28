use std::io::BufRead;

use clap::Args;
use serde::Deserialize;

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
  /// The task ID or prefix (conflicts with `--batch`).
  #[arg(conflicts_with = "batch")]
  task: Option<String>,
  /// Read NDJSON task records from stdin (one JSON object per line).
  #[arg(long, conflicts_with_all = ["task", "phase"])]
  batch: bool,
  /// The phase to add the task to (defaults to max existing + 1; conflicts with `--batch`).
  #[arg(long, short, conflicts_with = "batch")]
  phase: Option<u32>,
  #[command(flatten)]
  output: json::Flags,
}

/// A single task record in NDJSON batch mode.
#[derive(Debug, Deserialize)]
struct BatchRecord {
  /// Phase override (auto-increments from max when absent).
  phase: Option<u32>,
  /// Task ID or prefix to resolve.
  task: String,
}

impl Command {
  /// Attach the resolved task(s) to the iteration.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration add: entry");
    if self.batch {
      return self.call_batch(context).await;
    }

    let task_ref = self
      .task
      .as_ref()
      .ok_or_else(|| Error::Argument("task argument is required when --batch is not used".into()))?;

    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let iteration_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.iteration).await?;
    let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, task_ref).await?;

    let phase = match self.phase {
      Some(p) => p,
      None => {
        let max = repo::iteration::max_phase(&conn, &iteration_id).await?;
        max.map(|m| m + 1).unwrap_or(1)
      }
    };

    let tx = repo::transaction::begin(&conn, project_id, "iteration add").await?;
    repo::iteration::add_task(&conn, &iteration_id, &task_id, phase).await?;
    repo::transaction::record_event(&conn, tx.id(), "iteration_tasks", &task_id.to_string(), "created", None).await?;

    let short_id = task_id.short();
    let result = serde_json::json!({
      "task_id": task_id.to_string(),
      "iteration_id": iteration_id.to_string(),
      "phase": phase,
    });
    self.output.print_entity(&result, &short_id, || {
      log::info!("added task to iteration");
      SuccessMessage::new("added task to iteration")
        .field("task", task_id.short())
        .field("iteration", iteration_id.short())
        .field("phase", phase.to_string())
        .to_string()
    })?;
    Ok(())
  }

  async fn call_batch(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let iteration_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.iteration).await?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration batch add").await?;

    let mut next_phase = repo::iteration::max_phase(&conn, &iteration_id)
      .await?
      .map(|m| m + 1)
      .unwrap_or(1);

    let stdin = std::io::stdin().lock();
    let mut count = 0u32;

    for line in stdin.lines() {
      let line = line?;
      let trimmed = line.trim();
      if trimmed.is_empty() {
        continue;
      }

      let record: BatchRecord =
        serde_json::from_str(trimmed).map_err(|e| Error::Argument(format!("invalid NDJSON: {e}")))?;

      let task_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &record.task).await?;

      let phase = match record.phase {
        Some(p) => p,
        None => {
          let p = next_phase;
          next_phase += 1;
          p
        }
      };

      repo::iteration::add_task(&conn, &iteration_id, &task_id, phase).await?;
      repo::transaction::record_event(&conn, tx.id(), "iteration_tasks", &task_id.to_string(), "created", None).await?;

      count += 1;
    }

    log::info!("batch added {count} tasks to iteration");
    let short_id = iteration_id.short();
    let result = serde_json::json!({
      "iteration_id": iteration_id.to_string(),
      "count": count,
    });
    self.output.print_entity(&result, &short_id, || {
      SuccessMessage::new("batch added tasks to iteration")
        .field("count", count.to_string())
        .to_string()
    })?;
    Ok(())
  }
}
