use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{repo, search_query},
  ui::components::SearchResults,
};

/// Search across tasks, artifacts, and iterations.
#[derive(Args, Debug)]
pub struct Command {
  /// The search term.
  query: String,
  /// Show full content without truncation.
  #[arg(short, long)]
  expand: bool,
  /// Emit results as JSON.
  #[arg(short, long)]
  json: bool,
  /// Include resolved/archived entities.
  #[arg(short = 'a', long = "all")]
  show_all: bool,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let parsed = search_query::parse(&self.query);
    let results = repo::search::query(&conn, project_id, &parsed, self.show_all).await?;

    if self.json {
      let json_value = serde_json::json!({
        "query": self.query,
        "tasks": results.tasks,
        "artifacts": results.artifacts,
        "iterations": results.iterations,
      });
      let json = serde_json::to_string_pretty(&json_value)?;
      println!("{json}");
      return Ok(());
    }

    let view = SearchResults::new(self.query.clone(), results.tasks, results.artifacts, results.iterations)
      .expanded(self.expand);
    println!("{view}");

    Ok(())
  }
}
