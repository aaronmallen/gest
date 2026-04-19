use std::io::{BufRead, IsTerminal};

use clap::Args;
use serde::Deserialize;
use serde_json::Value;

use crate::{
  AppContext,
  cli::{Error, commands::helpers::extract_heading, meta_args, tag_arg},
  store::{
    model::{
      artifact::New,
      primitives::{EntityType, RelationshipType},
    },
    repo,
  },
  ui::{components::SuccessMessage, envelope::Envelope, json},
};

/// Create a new artifact.
///
/// When stdin is piped, the first markdown heading (`# …`) is used as the title
/// and the full input becomes the body. A title can also be given as a positional
/// argument, in which case stdin (if piped) is used as the body only.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact title (extracted from the first `# heading` when piping stdin).
  title: Option<String>,
  /// Read NDJSON artifacts from stdin (one JSON object per line).
  #[arg(long, conflicts_with_all = ["title", "body", "iteration", "metadata", "metadata_json", "source", "tag"])]
  batch: bool,
  /// The artifact body (markdown; opens `$EDITOR` if omitted and stdin is a terminal).
  #[arg(long, short)]
  body: Option<String>,
  /// Link the artifact to an iteration by ID or prefix.
  #[arg(long, short)]
  iteration: Option<String>,
  /// Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference).
  #[arg(long = "metadata", short = 'm', value_name = "KEY=VALUE")]
  metadata: Vec<String>,
  /// Merge a JSON object into metadata (repeatable; applied after --metadata pairs).
  #[arg(long = "metadata-json", value_name = "JSON")]
  metadata_json: Vec<String>,
  /// Read the body from a file path instead of stdin or `$EDITOR`.
  #[arg(long, short, conflicts_with = "body")]
  source: Option<String>,
  /// Add a tag to the artifact (can be repeated; comma-separated values split into multiple tags).
  #[arg(long, short, value_delimiter = ',', value_parser = tag_arg::trim_tag)]
  tag: Vec<String>,
  #[command(flatten)]
  output: json::Flags,
}

/// A single artifact record in NDJSON batch mode.
#[derive(Debug, Deserialize)]
struct BatchRecord {
  #[serde(default)]
  body: Option<String>,
  #[serde(default)]
  iteration: Option<String>,
  #[serde(default)]
  metadata: Option<Value>,
  #[serde(default)]
  tags: Option<Vec<String>>,
  title: String,
}

impl Command {
  /// Insert a new artifact (or a batch from NDJSON stdin), attaching tags and any iteration link.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact create: entry");
    if self.batch {
      return self.call_batch(context).await;
    }

    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let (title, body) = self.resolve_title_and_body()?;
    let metadata = self.parse_metadata()?;

    let new = New {
      body,
      metadata,
      title,
    };

    let tx = repo::transaction::begin(&conn, project_id, "artifact create").await?;
    let artifact = repo::artifact::create(&conn, project_id, &new).await?;
    repo::transaction::record_semantic_event(
      &conn,
      tx.id(),
      "artifacts",
      &artifact.id().to_string(),
      "created",
      None,
      Some("created"),
      None,
      None,
    )
    .await?;

    for label in tag_arg::normalize_tags(&self.tag) {
      let tag = repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), &label).await?;
      repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None).await?;
    }

    if let Some(iteration_ref) = &self.iteration {
      let iteration_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, iteration_ref).await?;
      let rel = repo::relationship::create(
        &conn,
        RelationshipType::RelatesTo,
        EntityType::Artifact,
        artifact.id(),
        EntityType::Iteration,
        &iteration_id,
      )
      .await?;
      repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None).await?;
    }

    let id_str = artifact.id().to_string();
    let prefix_lens = repo::artifact::prefix_lengths(&conn, project_id, &[id_str.as_str()]).await?;
    let prefix_len = prefix_lens[0];
    let short_id = artifact.id().short();
    log::info!("created artifact {short_id}");
    let envelope = Envelope::load_one(&conn, EntityType::Artifact, artifact.id(), &artifact, true).await?;
    self.output.print_envelope(&envelope, &short_id, || {
      SuccessMessage::new("created artifact")
        .id(artifact.id().short())
        .prefix_len(prefix_len)
        .field("title", artifact.title().to_string())
        .to_string()
    })?;
    Ok(())
  }

  async fn call_batch(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;
    let tx = repo::transaction::begin(&conn, project_id, "artifact batch create").await?;

    let stdin = std::io::stdin().lock();
    let mut count = 0u32;

    for line in stdin.lines() {
      let line = line?;
      let trimmed = line.trim();
      if trimmed.is_empty() {
        continue;
      }

      let record: BatchRecord =
        serde_json::from_str(trimmed).map_err(|e| Error::Editor(format!("invalid NDJSON: {e}")))?;

      let new = New {
        body: record.body.unwrap_or_default(),
        metadata: record.metadata,
        title: record.title,
      };

      let artifact = repo::artifact::create(&conn, project_id, &new).await?;
      repo::transaction::record_semantic_event(
        &conn,
        tx.id(),
        "artifacts",
        &artifact.id().to_string(),
        "created",
        None,
        Some("created"),
        None,
        None,
      )
      .await?;

      if let Some(tags) = &record.tags {
        for label in tags {
          let tag = repo::tag::attach(&conn, EntityType::Artifact, artifact.id(), label).await?;
          repo::transaction::record_event(&conn, tx.id(), "entity_tags", &tag.id().to_string(), "created", None)
            .await?;
        }
      }

      if let Some(iteration_ref) = &record.iteration {
        let iteration_id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Iterations, iteration_ref).await?;
        let rel = repo::relationship::create(
          &conn,
          RelationshipType::RelatesTo,
          EntityType::Artifact,
          artifact.id(),
          EntityType::Iteration,
          &iteration_id,
        )
        .await?;
        repo::transaction::record_event(&conn, tx.id(), "relationships", &rel.id().to_string(), "created", None)
          .await?;
      }

      count += 1;
    }

    log::info!("batch created {count} artifacts");
    let message = SuccessMessage::new("batch created artifacts").field("count", count.to_string());
    println!("{message}");
    Ok(())
  }

  fn parse_metadata(&self) -> Result<Option<Value>, Error> {
    meta_args::build_metadata(None, &self.metadata, &self.metadata_json)
  }

  fn resolve_title_and_body(&self) -> Result<(String, String), Error> {
    if let Some(title) = &self.title {
      // Title given explicitly — body from --source, --body, stdin, or editor
      let body = if let Some(path) = &self.source {
        std::fs::read_to_string(path).map_err(|e| Error::Editor(format!("failed to read source file: {e}")))?
      } else if let Some(b) = &self.body {
        b.clone()
      } else if std::io::stdin().is_terminal() {
        crate::io::editor::edit_text_with_suffix("", ".md").map_err(|e| Error::Editor(e.to_string()))?
      } else {
        std::io::read_to_string(std::io::stdin()).unwrap_or_default()
      };
      Ok((title.clone(), body))
    } else if !std::io::stdin().is_terminal() {
      // No title arg, stdin is piped — parse title from first heading
      let input = std::io::read_to_string(std::io::stdin()).unwrap_or_default();
      let title = extract_heading(&input)
        .ok_or_else(|| Error::Editor("no title provided and no # heading found in stdin".into()))?;
      Ok((title, input))
    } else {
      Err(Error::Editor("artifact title is required".into()))
    }
  }
}
