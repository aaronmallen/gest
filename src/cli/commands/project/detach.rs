use std::io::{Error as IoError, ErrorKind};

use clap::Args;

use crate::{AppContext, cli::Error, store::repo, ui::components::SuccessMessage};

/// Detach the current directory from its project.
#[derive(Args, Debug)]
pub struct Command;

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let cwd = std::env::current_dir()?;
    let conn = context.store().connect().await?;

    let detached = repo::project::detach_workspace(&conn, &cwd).await?;

    if !detached {
      return Err(IoError::new(ErrorKind::NotFound, format!("no workspace found for {}", cwd.display())).into());
    }

    let message = SuccessMessage::new("detached workspace").field("path", cwd.display().to_string());

    println!("{message}");
    Ok(())
  }
}
