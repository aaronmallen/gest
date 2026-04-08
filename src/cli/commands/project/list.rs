use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::components::{ProjectEntry, ProjectListView, min_unique_prefix},
};

/// List all known projects.
#[derive(Args, Debug)]
pub struct Command {
  /// Emit output as JSON.
  #[arg(long)]
  json: bool,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let projects = repo::project::all(&conn).await?;

    if self.json {
      let json = serde_json::to_string_pretty(&projects)?;
      println!("{json}");
      return Ok(());
    }

    let id_shorts: Vec<String> = projects.iter().map(|p| p.id().short()).collect();
    let prefix_len = {
      let refs: Vec<&str> = id_shorts.iter().map(String::as_str).collect();
      min_unique_prefix(&refs)
    };

    let entries: Vec<ProjectEntry> = projects
      .iter()
      .zip(id_shorts.iter())
      .map(|(project, id_short)| ProjectEntry {
        id: id_short.clone(),
        root: project.root().display().to_string(),
      })
      .collect();

    crate::io::pager::page(&format!("{}\n", ProjectListView::new(entries, prefix_len)), context)?;

    Ok(())
  }
}
