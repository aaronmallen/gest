use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::primitives::{EntityType, RelationshipType},
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Mark a task as blocking another task (shortcut for `task link <id> <blocked> --rel blocks`).
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix (the blocker).
  id: String,
  /// The task being blocked.
  blocked: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let target_id = repo::resolve::resolve_id(&conn, "tasks", &self.blocked).await?;

    let tx = repo::transaction::begin(&conn, project_id, "task block").await?;
    let rel = repo::relationship::create(
      &conn,
      RelationshipType::Blocks,
      EntityType::Task,
      &source_id,
      EntityType::Task,
      &target_id,
    )
    .await?;
    repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;

    let short_id = source_id.short();
    self.output.print_entity(&rel, &short_id, || {
      SuccessMessage::new("linked task")
        .id(source_id.short())
        .field("rel", "blocks")
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
