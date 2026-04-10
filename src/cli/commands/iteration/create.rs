use clap::Args;

use crate::{
  AppContext,
  cli::{Error, meta_args, tag_arg},
  store::{
    model::{
      iteration::New,
      primitives::{EntityType, IterationStatus},
    },
    repo,
  },
  ui::{components::SuccessMessage, json},
};

/// Create a new iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration title.
  title: String,
  /// The iteration description.
  #[arg(long, short)]
  description: Option<String>,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// Set the initial status.
  #[arg(long, short)]
  status: Option<String>,
  /// Add a tag (may be repeated; comma-separated values split into multiple tags).
  #[arg(long, short, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tag: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Insert a new iteration with optional initial status and tags within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("iteration create: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let metadata = meta_args::build_metadata(None, &self.metadata, &self.metadata_json)?;

    let new = New {
      description: self.description.clone().unwrap_or_default(),
      metadata,
      title: self.title.clone(),
    };

    let tx = repo::transaction::begin(&conn, project_id, "iteration create").await?;
    let iteration = repo::iteration::create(&conn, project_id, &new).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "iterations",
      &iteration.id().to_string(),
      "created",
      None,
      Some("created"),
      None,
      None,
    )
    .await?;

    // Apply initial status if provided
    if let Some(status_str) = &self.status {
      let status = status_str.parse().map_err(|e: String| Error::Argument(e))?;
      let patch = crate::store::model::iteration::Patch {
        status: Some(status),
        ..Default::default()
      };
      repo::iteration::update(&conn, iteration.id(), &patch).await?;
    }

    // Apply tags
    for label in tag_arg::normalize_tags(&self.tag) {
      repo::tag::attach(&conn, EntityType::Iteration, iteration.id(), &label).await?;
    }

    let prefix_len = match iteration.status() {
      IterationStatus::Completed | IterationStatus::Cancelled => {
        repo::iteration::shortest_all_prefix(&conn, project_id).await?
      }
      _ => repo::iteration::shortest_active_prefix(&conn, project_id).await?,
    };

    let short_id = iteration.id().short();
    log::info!("created iteration {short_id}");
    self.output.print_entity(&iteration, &short_id, || {
      SuccessMessage::new("created iteration")
        .id(iteration.id().short())
        .prefix_len(prefix_len)
        .field("title", iteration.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
