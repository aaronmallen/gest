use clap::Args;

use crate::{
  AppContext,
  cli::{Error, limit::LimitArgs},
  store::repo,
  ui::{
    components::{ProjectEntry, ProjectListView, min_unique_prefix},
    json,
  },
};

/// List all known projects.
#[derive(Args, Debug)]
pub struct Command {
  #[command(flatten)]
  limit: LimitArgs,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project list: entry");
    let conn = context.store().connect().await?;
    let mut projects = repo::project::all(&conn).await?;
    self.limit.apply(&mut projects);

    let id_shorts: Vec<String> = projects.iter().map(|p| p.id().short()).collect();

    if self.output.json {
      let json = serde_json::to_string_pretty(&projects)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for id in &id_shorts {
        println!("{id}");
      }
      return Ok(());
    }

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
