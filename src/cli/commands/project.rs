//! `gest project` subcommand tree for showing, archiving, attaching, and listing projects.

mod archive;
mod attach;
mod delete;
mod detach;
mod list;
mod unarchive;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error, store::repo, ui::components::ProjectShow};

/// Show or manage the current project.
#[derive(Args, Debug)]
pub struct Command {
  /// Emit output as JSON (only applies to the default show view).
  #[arg(long)]
  json: bool,
  #[command(subcommand)]
  subcommand: Option<Sub>,
}

#[derive(Debug, Subcommand)]
enum Sub {
  /// Soft-archive a project, hiding it and its entities from default list views.
  Archive(archive::Command),
  /// Attach the current directory to an existing project as a workspace.
  Attach(attach::Command),
  /// Delete a project and all of its owned entities.
  #[command(visible_alias = "rm")]
  Delete(delete::Command),
  /// Detach the current directory from its project.
  Detach(detach::Command),
  /// List all known projects.
  #[command(visible_alias = "ls")]
  List(list::Command),
  /// Restore an archived project to active status.
  Unarchive(unarchive::Command),
}

impl Command {
  /// Dispatch to the matched project subcommand, or show the current project.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("project: entry");
    match &self.subcommand {
      Some(Sub::Archive(command)) => command.call(context).await,
      Some(Sub::Attach(command)) => command.call(context).await,
      Some(Sub::Delete(command)) => command.call(context).await,
      Some(Sub::Detach(command)) => command.call(context).await,
      Some(Sub::List(command)) => command.call(context).await,
      Some(Sub::Unarchive(command)) => command.call(context).await,
      None => self.show(context).await,
    }
  }

  async fn show(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let project = repo::project::find_by_id(&conn, project_id.clone())
      .await?
      .ok_or_else(|| Error::NotFound(format!("project not found: {project_id}")))?;

    if self.json {
      let json = serde_json::to_string_pretty(&project)?;
      println!("{json}");
      return Ok(());
    }

    let view = ProjectShow::new(project.id().short(), project.root().display().to_string());
    println!("{view}");
    Ok(())
  }
}
