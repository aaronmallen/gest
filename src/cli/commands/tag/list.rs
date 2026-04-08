use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::json,
};

/// List all tags, optionally filtered by entity type.
#[derive(Args, Debug, Default)]
pub struct Command {
  /// Show only tags attached to artifacts.
  #[arg(long, conflicts_with_all = ["task", "iteration"])]
  artifact: bool,
  /// Show only tags attached to iterations.
  #[arg(long, conflicts_with_all = ["task", "artifact"])]
  iteration: bool,
  /// Show only tags attached to tasks.
  #[arg(long, conflicts_with_all = ["artifact", "iteration"])]
  task: bool,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let tags = match self.entity_type_filter() {
      Some(entity_type) => repo::tag::by_entity_type(&conn, entity_type).await?,
      None => repo::tag::all(&conn).await?,
    };

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
      crate::io::pager::page("  no tags\n", context)?;
      return Ok(());
    }

    let mut output = String::new();
    for tag in &tags {
      use std::fmt::Write;
      let _ = writeln!(output, "  #{}", tag.label());
    }
    crate::io::pager::page(&output, context)?;

    Ok(())
  }

  fn entity_type_filter(&self) -> Option<EntityType> {
    if self.task {
      Some(EntityType::Task)
    } else if self.artifact {
      Some(EntityType::Artifact)
    } else if self.iteration {
      Some(EntityType::Iteration)
    } else {
      None
    }
  }
}
