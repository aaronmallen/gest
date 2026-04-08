use clap::Args;

use crate::{
  AppContext,
  cli::{Error, limit::LimitArgs},
  store::{model::primitives::EntityType, repo},
  ui::{components::FieldList, json},
};

/// List notes on a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  #[command(flatten)]
  limit: LimitArgs,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let conn = context.store().connect().await?;
    let task_id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;

    let mut notes = repo::note::for_entity(&conn, EntityType::Task, &task_id).await?;
    self.limit.apply(&mut notes);

    if self.output.json {
      let json = serde_json::to_string_pretty(&notes)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for note in &notes {
        println!("{}", note.id().short());
      }
      return Ok(());
    }

    if notes.is_empty() {
      crate::io::pager::page("  no notes\n", context)?;
      return Ok(());
    }

    use std::fmt::Write;
    let mut output = String::new();
    for (i, note) in notes.iter().enumerate() {
      if i > 0 {
        output.push('\n');
      }
      let fields = FieldList::new()
        .field("id", note.id().short())
        .field("body", note.body().to_string());
      let _ = writeln!(output, "{fields}");
    }
    crate::io::pager::page(&output, context)?;

    Ok(())
  }
}
