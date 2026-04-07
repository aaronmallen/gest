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

/// Link an iteration to another entity.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// The relationship type (e.g. blocks, blocked-by, relates-to).
  rel: RelationshipType,
  /// The target entity ID or prefix.
  target: String,
  /// Target is an artifact instead of another iteration.
  #[arg(long)]
  artifact: bool,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, "iterations", &self.id).await?;
    let (target_type, target_table) = if self.artifact {
      (EntityType::Artifact, "artifacts")
    } else {
      (EntityType::Iteration, "iterations")
    };
    let target_id = repo::resolve::resolve_id(&conn, target_table, &self.target).await?;

    let tx = repo::transaction::begin(&conn, project_id, "iteration link").await?;
    let rel = repo::relationship::create(
      &conn,
      self.rel,
      EntityType::Iteration,
      &source_id,
      target_type,
      &target_id,
    )
    .await?;
    repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;

    // Write reciprocal link for iteration-to-iteration relationships.
    if !self.artifact {
      let inverse = repo::relationship::create(
        &conn,
        self.rel.inverse(),
        EntityType::Iteration,
        &target_id,
        EntityType::Iteration,
        &source_id,
      )
      .await?;
      repo::transaction::record_event(
        &conn,
        tx.id(),
        "relationships",
        &inverse.id().to_string(),
        "created",
        None,
      )
      .await?;
    }

    let short_id = source_id.short();
    self.output.print_entity(&rel, &short_id, || {
      SuccessMessage::new("linked iteration")
        .id(source_id.short())
        .field("rel", self.rel.to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
