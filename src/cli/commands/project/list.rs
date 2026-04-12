use clap::Args;

use crate::{
  AppContext,
  cli::{Error, limit::LimitArgs},
  store::repo,
  ui::{
    components::{ProjectEntry, ProjectListView, unique_prefix_lengths},
    json,
  },
};

/// List all known projects.
#[derive(Args, Debug)]
pub struct Command {
  /// Show all projects, including archived.
  #[arg(long, short)]
  all: bool,
  #[command(flatten)]
  limit: LimitArgs,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Render all known projects, honoring the shared limit and output flags.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project list: entry");
    let conn = context.store().connect().await?;
    let mut projects = repo::project::all(&conn, self.all).await?;
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

    let prefix_lens = {
      let refs: Vec<&str> = id_shorts.iter().map(String::as_str).collect();
      unique_prefix_lengths(&refs)
    };

    let entries: Vec<ProjectEntry> = projects
      .iter()
      .zip(id_shorts.iter())
      .zip(prefix_lens.iter())
      .map(|((project, id_short), &prefix_len)| ProjectEntry {
        archived: project.archived_at().is_some(),
        id: id_short.clone(),
        prefix_len,
        root: project.root().display().to_string(),
      })
      .collect();

    crate::io::pager::page(&format!("{}\n", ProjectListView::new(entries)), context)?;

    Ok(())
  }
}
