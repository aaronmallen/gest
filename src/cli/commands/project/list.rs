use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::repo,
  ui::components::{EmptyList, ProjectListRow},
};

/// List all known projects.
#[derive(Args, Debug)]
pub struct Command;

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let projects = repo::project::all(&conn).await?;

    if projects.is_empty() {
      println!("{}", EmptyList::new("projects"));
      return Ok(());
    }

    for (i, project) in projects.iter().enumerate() {
      if i > 0 {
        println!();
      }
      let row = ProjectListRow::new(project.id().short(), project.root().display().to_string());
      println!("{row}");
    }

    Ok(())
  }
}
