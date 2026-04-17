use std::str::FromStr;

use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, Prefixable},
  cli::Error,
  store::{
    model::primitives::{EntityType, RelationshipType},
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Link an iteration to another entity.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Positional arguments: either `<target>` (new form) or `<rel> <target>` (deprecated).
  #[arg(value_name = "[REL] TARGET", num_args = 1..=2)]
  args: Vec<String>,
  /// Target is an artifact instead of another iteration.
  #[arg(long)]
  artifact: bool,
  /// The relationship type (e.g. blocks, blocked-by, relates-to).
  #[arg(long = "rel")]
  rel: Option<RelationshipType>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Create a relationship row (and reciprocal for iteration-to-iteration links) within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration link: entry");
    let (rel, target) = self.resolve_rel_and_target()?;

    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let source_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let (target_type, target_table) = if self.artifact {
      (EntityType::Artifact, repo::resolve::Table::Artifacts)
    } else {
      (EntityType::Iteration, repo::resolve::Table::Iterations)
    };
    let target_id = repo::resolve::resolve_id(&conn, target_table, &target).await?;

    let tx = repo::transaction::begin(&conn, project_id, "iteration link").await?;
    let relationship =
      repo::relationship::create(&conn, rel, EntityType::Iteration, &source_id, target_type, &target_id).await?;
    repo::transaction::record_event(
      &conn,
      tx.id(),
      "relationships",
      &relationship.id().to_string(),
      "created",
      None,
    )
    .await?;

    // Write reciprocal link for iteration-to-iteration relationships.
    if !self.artifact {
      let inverse = repo::relationship::create(
        &conn,
        rel.inverse(),
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

    let iteration = repo::iteration::find_required_by_id(&conn, source_id.clone()).await?;
    let prefix_len = Iteration::prefix_length(&conn, project_id, &iteration.id().to_string()).await?;
    let short_id = source_id.short();
    let envelope = Envelope::load_one(&conn, EntityType::Iteration, &source_id, &iteration, true).await?;
    self.output.print_envelope(&envelope, &short_id, || {
      SuccessMessage::new("linked iteration")
        .id(source_id.short())
        .prefix_len(prefix_len)
        .field("rel", rel.to_string())
        .field("target", target_id.short())
        .to_string()
    })?;
    Ok(())
  }

  /// Resolve the effective relationship type and target string from positional args and flags.
  ///
  /// Supports the new `<id> <target> [--rel <type>]` form and the legacy
  /// `<id> <rel> <target>` form. The legacy form emits a deprecation warning to stderr,
  /// and combining a positional rel with `--rel` is rejected as a conflict.
  fn resolve_rel_and_target(&self) -> Result<(RelationshipType, String), Error> {
    match self.args.as_slice() {
      [target] => {
        let rel = self.rel.unwrap_or(RelationshipType::RelatesTo);
        Ok((rel, target.clone()))
      }
      [positional_rel, target] => {
        if self.rel.is_some() {
          return Err(Error::Argument(
            "cannot specify rel both as a positional argument and via --rel".to_string(),
          ));
        }
        eprintln!(
          "warning: passing <rel> as a positional argument is deprecated; use --rel <type>. This form will be \
          removed in a future release."
        );
        let rel = RelationshipType::from_str(positional_rel).map_err(Error::Argument)?;
        Ok((rel, target.clone()))
      }
      _ => Err(Error::Argument(
        "iteration link requires <target> or <rel> <target>".to_string(),
      )),
    }
  }
}
