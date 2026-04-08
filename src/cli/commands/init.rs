//! The `init` subcommand — initializes a new gest project.

use clap::Args;

use crate::{AppContext, cli::Error, store::repo, ui::components::SuccessMessage};

/// Initialize gest for the current directory.
#[derive(Args, Debug)]
pub struct Command {
  /// Create a `.gest` directory in the current project with a local `project.json`
  /// instead of using the global data store.
  #[arg(long)]
  local: bool,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("init: entry");
    let cwd = std::env::current_dir()?;

    if self.local {
      std::fs::create_dir_all(cwd.join(".gest"))?;
    }

    let conn = context.store().connect().await?;
    let project = repo::project::find_by_path(&conn, &cwd).await?;

    let project = match project {
      Some(project) => project,
      None => repo::project::create(&conn, &cwd).await?,
    };

    let mut message = SuccessMessage::new("initialized project").id(project.id().short());
    message = message.field("root", cwd.display().to_string());

    if self.local {
      message = message.field("gest dir", ".gest/");
    }

    println!("{message}");
    Ok(())
  }
}
