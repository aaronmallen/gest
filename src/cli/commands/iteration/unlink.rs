use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, Prefixable},
  cli::Error,
  store::{
    Error as StoreError,
    model::{
      primitives::{EntityType, RelationshipType},
      relationship::Model as Relationship,
    },
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Remove a relationship between an iteration and another entity.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// The target entity ID or prefix.
  target: String,
  /// Target is an artifact instead of another iteration.
  #[arg(long)]
  artifact: bool,
  /// The relationship type (e.g. blocks, blocked-by, relates-to). Required when multiple edges exist.
  #[arg(long = "rel")]
  rel: Option<RelationshipType>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Delete the matching relationship row (and its reciprocal for iteration-to-iteration edges) within a recorded
  /// transaction so the operation can be undone atomically.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration unlink: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let (target_type, target_table) = if self.artifact {
      (EntityType::Artifact, repo::resolve::Table::Artifacts)
    } else {
      (EntityType::Iteration, repo::resolve::Table::Iterations)
    };
    let target_id = repo::resolve::resolve_id(&conn, target_table, &self.target).await?;

    let matches = repo::relationship::find_by_endpoints(
      &conn,
      EntityType::Iteration,
      &source_id,
      target_type,
      &target_id,
      self.rel,
    )
    .await?;

    let matched = match matches.as_slice() {
      [] => {
        let suffix = self.rel.map(|r| format!(" with rel-type {r}")).unwrap_or_default();
        return Err(Error::NotFound(format!(
          "no relationship found between {} and {}{}",
          source_id.short(),
          target_id.short(),
          suffix
        )));
      }
      [single] => single.clone(),
      many => {
        let rels = many
          .iter()
          .map(|r| r.rel_type().to_string())
          .collect::<Vec<_>>()
          .join(", ");
        return Err(Error::Argument(format!(
          "multiple relationships found between {} and {}; specify --rel <type> to disambiguate ({})",
          source_id.short(),
          target_id.short(),
          rels
        )));
      }
    };

    let reciprocal = if target_type == EntityType::Iteration {
      let rows = repo::relationship::find_by_endpoints(
        &conn,
        EntityType::Iteration,
        &target_id,
        EntityType::Iteration,
        &source_id,
        Some(matched.rel_type().inverse()),
      )
      .await?;
      rows.into_iter().next()
    } else {
      None
    };

    let tx = repo::transaction::begin(&conn, project_id, "iteration unlink").await?;
    record_and_delete(&conn, tx.id(), &matched).await?;
    if let Some(inverse) = &reciprocal {
      record_and_delete(&conn, tx.id(), inverse).await?;
    }

    let iteration = repo::iteration::find_required_by_id(&conn, source_id.clone()).await?;
    let prefix_len = Iteration::prefix_length(&conn, project_id, &iteration.id().to_string()).await?;
    let short_id = source_id.short();
    let envelope = Envelope::load_one(&conn, EntityType::Iteration, &source_id, &iteration, true).await?;
    self.output.print_envelope(&envelope, &short_id, || {
      SuccessMessage::new("unlinked iteration")
        .id(source_id.short())
        .prefix_len(prefix_len)
        .field("rel", matched.rel_type().to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }
}

/// Record a `deleted` event for `rel` and then remove the row.
///
/// The audit payload mirrors the on-disk `relationships` schema so `transaction::undo` can re-insert the row.
async fn record_and_delete(
  conn: &libsql::Connection,
  transaction_id: &crate::store::model::primitives::Id,
  rel: &Relationship,
) -> Result<(), StoreError> {
  let payload = repo::relationship::relationship_audit_payload(rel);
  repo::transaction::record_event(
    conn,
    transaction_id,
    "relationships",
    &rel.id().to_string(),
    "deleted",
    Some(&payload),
  )
  .await?;
  repo::relationship::delete(conn, rel.id()).await?;
  Ok(())
}
