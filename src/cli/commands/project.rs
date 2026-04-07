mod attach;
mod detach;
mod list;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error, store::repo, ui::components::ProjectShow};

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
  /// Attach the current directory to an existing project as a workspace.
  Attach(attach::Command),
  /// Detach the current directory from its project.
  Detach(detach::Command),
  /// List all known projects.
  List(list::Command),
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    match &self.subcommand {
      Some(Sub::Attach(command)) => command.call(context).await,
      Some(Sub::Detach(command)) => command.call(context).await,
      Some(Sub::List(command)) => command.call(context).await,
      None => self.show(context).await,
    }
  }

  async fn show(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let project = repo::project::find_by_id(&conn, project_id.clone())
      .await?
      .ok_or(Error::UninitializedProject)?;

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
