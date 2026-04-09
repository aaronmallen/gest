use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::{
    components::{GraphTask, IterationGraphView},
    json,
  },
};

/// Show phased task dependency graph.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Render the iteration's tasks grouped by phase as a dependency graph view.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration graph: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_required_by_id(&conn, id.clone()).await?;

    let tasks = repo::iteration::tasks_with_phase(&conn, &id).await?;

    if self.output.json {
      let json_tasks: Vec<_> = tasks
        .iter()
        .map(|t| {
          serde_json::json!({
            "blocked_by": t.blocked_by,
            "id": t.id_short,
            "is_blocking": t.is_blocking,
            "phase": t.phase,
            "priority": t.priority,
            "status": t.status,
            "title": t.title,
          })
        })
        .collect();
      let json = serde_json::json!({
        "id": id.to_string(),
        "title": iteration.title(),
        "tasks": json_tasks,
      });
      println!("{}", serde_json::to_string_pretty(&json)?);
      return Ok(());
    }

    if self.output.quiet {
      println!("{}", id.short());
      return Ok(());
    }

    let graph_tasks: Vec<GraphTask> = tasks
      .iter()
      .map(|t| GraphTask {
        blocked_by: t.blocked_by.clone(),
        id_short: t.id_short.clone(),
        is_blocking: t.is_blocking,
        phase: t.phase,
        priority: t.priority,
        status: t.status.clone(),
        title: t.title.clone(),
      })
      .collect();

    let task_prefix_len = repo::task::shortest_all_prefix(&conn, project_id).await?;

    print!(
      "{}",
      IterationGraphView::new(iteration.title(), graph_tasks).prefix_len(task_prefix_len)
    );

    Ok(())
  }
}
