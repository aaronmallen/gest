use clap::Args;

use crate::{
  AppContext,
  cli::{Error, prompt},
  store::repo,
  ui::components::SuccessMessage,
};

/// Soft-archive a project, detaching all workspaces and hiding owned entities from list views.
#[derive(Args, Debug)]
pub struct Command {
  /// The project ID or prefix.
  id: String,
  /// Skip the interactive confirmation prompt.
  #[arg(long)]
  yes: bool,
}

impl Command {
  /// Confirm and archive the project, detaching all workspace paths.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project archive: entry");
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Projects, &self.id).await?;
    let project = repo::project::find_by_id(&conn, id)
      .await?
      .ok_or_else(|| Error::Argument(format!("project {} not found", self.id)))?;

    let counts = repo::project::entity_counts(&conn, project.id()).await?;

    let target = format!(
      "project {} ({}). This will detach {} workspace paths and hide {} tasks, {} iterations, {} artifacts from list views",
      project.id().short(),
      project.root().display(),
      counts.workspaces,
      counts.tasks,
      counts.iterations,
      counts.artifacts,
    );
    if !prompt::confirm_destructive("archive", &target, self.yes)? {
      log::info!("project archive: aborted by user");
      return Ok(());
    }

    repo::project::archive(&conn, project.id()).await?;

    let message = SuccessMessage::new("archived project")
      .id(project.id().short())
      .field("root", project.root().display().to_string())
      .field("workspaces detached", counts.workspaces.to_string())
      .field("tasks hidden", counts.tasks.to_string())
      .field("iterations hidden", counts.iterations.to_string())
      .field("artifacts hidden", counts.artifacts.to_string());

    println!("{message}");
    Ok(())
  }
}
