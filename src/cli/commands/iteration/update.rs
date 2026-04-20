use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, Prefixable},
  cli::{Error, meta_args},
  store::{
    model::{iteration::Patch, primitives::EntityType},
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Update an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  /// Set the iteration description.
  #[arg(long, short)]
  description: Option<String>,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// Set the iteration title.
  #[arg(long, short)]
  title: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Apply title, description, and metadata changes to the resolved iteration within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration update: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, &self.id).await?;
    let before_iter = repo::iteration::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::NotFound(format!("iteration not found: {}", self.id)))?;

    let metadata = if self.metadata.is_empty() && self.metadata_json.is_empty() {
      None
    } else {
      meta_args::build_metadata(
        Some(before_iter.metadata().clone()),
        &self.metadata,
        &self.metadata_json,
      )?
    };

    let before = serde_json::to_value(&before_iter)?;
    let tx = repo::transaction::begin(&conn, project_id, "iteration update").await?;
    let patch = Patch {
      description: self.description.clone(),
      metadata,
      title: self.title.clone(),
      ..Default::default()
    };

    let iteration = repo::iteration::update(&conn, &id, &patch).await?;
    repo::transaction::record_event(&conn, tx.id(), "iterations", &id.to_string(), "modified", Some(&before)).await?;

    let prefix_len = Iteration::prefix_length(&conn, project_id, &iteration.id().to_string()).await?;

    let short_id = iteration.id().short();
    let envelope = Envelope::load_one(&conn, EntityType::Iteration, &id, &iteration, true).await?;
    self.output.print_envelope(&envelope, &short_id, || {
      log::info!("updated iteration");
      SuccessMessage::new("updated iteration")
        .id(iteration.id().short())
        .prefix_len(prefix_len)
        .field("title", iteration.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
