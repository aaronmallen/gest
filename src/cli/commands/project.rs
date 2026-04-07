mod attach;
mod detach;
mod list;

use clap::{Args, Subcommand};

use crate::{AppContext, cli::Error, ui::components::Id};

#[derive(Args, Debug)]
pub struct Command {
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
      None => {
        let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
        println!("{}", Id::new(&project_id.to_string()));
        Ok(())
      }
    }
  }
}
