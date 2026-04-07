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

/// Link a task to another entity.
#[derive(Args, Debug)]
pub struct Command {
  /// Target is an artifact instead of a task.
  #[arg(long, conflicts_with = "target_type")]
  artifact: bool,
  /// The task ID or prefix.
  id: String,
  /// The relationship type.
  #[arg(long, short, default_value = "relates-to")]
  rel: RelationshipType,
  /// The target entity ID or prefix.
  target: String,
  /// The target entity type.
  #[arg(long, default_value = "task")]
  target_type: EntityType,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, "tasks", &self.id).await?;
    let target_type = if self.artifact {
      EntityType::Artifact
    } else {
      self.target_type
    };
    let target_table = match target_type {
      EntityType::Artifact => "artifacts",
      EntityType::Iteration => "iterations",
      EntityType::Task => "tasks",
    };
    let target_id = repo::resolve::resolve_id(&conn, target_table, &self.target).await?;

    let tx = repo::transaction::begin(&conn, project_id, "task link").await?;
    let rel =
      repo::relationship::create(&conn, self.rel, EntityType::Task, &source_id, target_type, &target_id).await?;
    repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;

    let short_id = source_id.short();
    self.output.print_entity(&rel, &short_id, || {
      SuccessMessage::new("linked task")
        .id(source_id.short())
        .field("rel", self.rel.to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
