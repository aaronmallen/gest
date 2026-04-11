use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::primitives::{EntityType, RelationshipType},
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
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
  #[arg(long, default_value = "relates-to")]
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
  /// Create a relationship row from the task to the resolved target entity within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task link: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
    let target_type = if self.artifact {
      EntityType::Artifact
    } else {
      self.target_type
    };
    let target_table = match target_type {
      EntityType::Artifact => repo::resolve::Table::Artifacts,
      EntityType::Iteration => repo::resolve::Table::Iterations,
      EntityType::Task => repo::resolve::Table::Tasks,
    };
    let target_id = repo::resolve::resolve_id(&conn, target_table, &self.target).await?;

    let tx = repo::transaction::begin(&conn, project_id, "task link").await?;
    let rel =
      repo::relationship::create(&conn, self.rel, EntityType::Task, &source_id, target_type, &target_id).await?;
    repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;

    // Pool follows the source task's status.
    let source_task = repo::task::find_required_by_id(&conn, source_id.clone()).await?;
    let envelope = Envelope::load_one(&conn, EntityType::Task, source_task.id(), &source_task, true).await?;

    let prefix_map = repo::task::per_id_prefix_lengths(&conn, project_id).await?;
    let prefix_len = prefix_map.get(&source_id.to_string()).copied().unwrap_or(1);

    let short_id = source_id.short();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("linked task");
      SuccessMessage::new("linked task")
        .id(source_id.short())
        .prefix_len(prefix_len)
        .field("rel", self.rel.to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
