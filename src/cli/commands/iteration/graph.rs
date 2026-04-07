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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let iteration = repo::iteration::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let tasks = repo::iteration::tasks_with_phase(&conn, &id).await?;

    if self.output.json {
      let json_tasks: Vec<_> = tasks
        .iter()
        .map(|t| {
          serde_json::json!({
            "id": t.id_short,
            "phase": t.phase,
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
        id_short: t.id_short.clone(),
        phase: t.phase,
        status: t.status.clone(),
        title: t.title.clone(),
      })
      .collect();

    print!("{}", IterationGraphView::new(iteration.title(), graph_tasks));

    Ok(())
  }
}
