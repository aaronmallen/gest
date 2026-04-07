use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{components::FieldList, json},
};

/// Show iteration progress summary.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let counts = repo::iteration::task_status_counts(&conn, &id).await?;
    let max_phase = repo::iteration::max_phase(&conn, &id).await?;
    let progress = if counts.total > 0 {
      (counts.done * 100) / counts.total
    } else {
      0
    };

    if self.output.json {
      let json = serde_json::json!({
        "id": id.to_string(),
        "title": iteration.title(),
        "status": iteration.status().to_string(),
        "phases": max_phase.unwrap_or(0),
        "total_tasks": counts.total,
        "open": counts.open,
        "in_progress": counts.in_progress,
        "done": counts.done,
        "cancelled": counts.cancelled,
        "progress": progress,
      });
      println!("{}", serde_json::to_string_pretty(&json)?);
      return Ok(());
    }

    if self.output.quiet {
      println!("{}", id.short());
      return Ok(());
    }

    let fields = FieldList::new()
      .field("iteration", iteration.title().to_string())
      .field("status", iteration.status().to_string())
      .field("phases", max_phase.unwrap_or(0).to_string())
      .field("total tasks", counts.total.to_string())
      .field("open", counts.open.to_string())
      .field("in progress", counts.in_progress.to_string())
      .field("done", counts.done.to_string())
      .field("cancelled", counts.cancelled.to_string())
      .field("progress", format!("{progress}%"));

    println!("{fields}");
    Ok(())
  }
}
