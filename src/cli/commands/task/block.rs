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
  /// Create a `blocks` relationship from the source task to the blocked task within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task block: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.id).await?;
    let target_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Tasks, &self.blocked).await?;

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
        .field("rel", "blocks")
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
