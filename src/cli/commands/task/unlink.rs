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

/// Remove a relationship between a task and another entity.
#[derive(Args, Debug)]
pub struct Command {
  /// Target is an artifact instead of a task.
  #[arg(long, conflicts_with = "target_type")]
  artifact: bool,
  /// The task ID or prefix.
  id: String,
  /// Filter to relationships of this type. Required when multiple
  /// relationships exist between the endpoints.
  #[arg(long)]
  rel: Option<RelationshipType>,
  /// The target entity ID or prefix.
  target: String,
  /// The target entity type.
  #[arg(long, default_value = "task")]
  target_type: EntityType,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Delete the matching relationship between the task and target within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("task unlink: entry");
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

    let matches =
      repo::relationship::find_by_endpoints(&conn, EntityType::Task, &source_id, target_type, &target_id, self.rel)
        .await?;

    let rel = match matches.len() {
      0 => {
        let rel_clause = match self.rel {
          Some(r) => format!(" with rel-type {r}"),
          None => String::new(),
        };
        return Err(Error::NotFound(format!(
          "no relationship found between {} and {}{}",
          source_id.short(),
          target_id.short(),
          rel_clause
        )));
      }
      1 => matches.into_iter().next().expect("length checked"),
      _ => {
        let candidates: Vec<String> = matches.iter().map(|r| r.rel_type().to_string()).collect();
        return Err(Error::Argument(format!(
          "multiple relationships found between {} and {}; specify --rel <type> to disambiguate (candidates: {})",
          source_id.short(),
          target_id.short(),
          candidates.join(", ")
        )));
      }
    };

    let payload = repo::relationship::relationship_audit_payload(&rel);

    let tx = repo::transaction::begin(&conn, project_id, "task unlink").await?;
    repo::transaction::record_event(
      &conn,
      tx.id(),
      "relationships",
      &rel.id().to_string(),
      "deleted",
      Some(&payload),
    )
    .await?;
    repo::relationship::delete(&conn, rel.id()).await?;

    // Pool follows the source task's status.
    let source_task = repo::task::find_required_by_id(&conn, source_id.clone()).await?;
    let envelope = Envelope::load_one(&conn, EntityType::Task, source_task.id(), &source_task, true).await?;

    let prefix_map = repo::task::per_id_prefix_lengths(&conn, project_id).await?;
    let prefix_len = prefix_map.get(&source_id.to_string()).copied().unwrap_or(1);

    let short_id = source_id.short();
    let rel_type = rel.rel_type();
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("unlinked task");
      SuccessMessage::new("unlinked task")
        .id(source_id.short())
        .prefix_len(prefix_len)
        .field("rel", rel_type.to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}
