use clap::Args;

use crate::{AppContext, cli::Error, store::repo, ui::json};

/// List all tags.
#[derive(Args, Debug, Default)]
pub struct Command {
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let tags = repo::tag::all(&conn).await?;

    if self.output.json {
      let json = serde_json::to_string_pretty(&tags)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for tag in &tags {
        println!("{}", tag.label());
      }
      return Ok(());
    }

    if tags.is_empty() {
      println!("  no tags");
      return Ok(());
    }

    for tag in &tags {
      println!("  #{}", tag.label());
    }

    Ok(())
  }
}
