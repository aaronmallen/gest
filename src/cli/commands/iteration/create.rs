use clap::Args;
use serde_json::Value;

use crate::{
  AppContext,
  cli::Error,
  store::{
    model::{iteration::New, primitives::EntityType},
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
  /// Set a metadata key=value pair (may be repeated).
  #[arg(long, short, value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Set the initial status.
  #[arg(long, short)]
  status: Option<String>,
  /// Add a tag (may be repeated).
  #[arg(long, short)]
  tag: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let metadata = if self.metadata.is_empty() {
      None
    } else {
      let mut map = serde_json::Map::new();
      for pair in &self.metadata {
        let (key, value) = pair
          .split_once('=')
          .ok_or_else(|| Error::Argument(format!("invalid metadata format: {pair} (expected KEY=VALUE)")))?;
        let parsed: Value = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()));
        map.insert(key.to_string(), parsed);
      }
      Some(Value::Object(map))
    };

    let new = New {
      description: self.description.clone().unwrap_or_default(),
      metadata,
      title: self.title.clone(),
    };

    let tx = repo::transaction::begin(&conn, project_id, "iteration create").await?;
    let iteration = repo::iteration::create(&conn, project_id, &new).await?;
    repo::transaction::record_event(
      &conn,
      tx.id(),
      "iterations",
      &iteration.id().to_string(),
      "created",
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
    for label in &self.tag {
      repo::tag::attach(&conn, EntityType::Iteration, iteration.id(), label).await?;
    }

    let short_id = iteration.id().short();
    self.output.print_entity(&iteration, &short_id, || {
      SuccessMessage::new("created iteration")
        .id(iteration.id().short())
        .field("title", iteration.title().to_string())
        .to_string()
    })?;
    Ok(())
  }
}
