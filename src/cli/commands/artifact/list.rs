use clap::Args;

use crate::{
  AppContext,
  cli::{Error, limit::LimitArgs},
  store::{
    model::{
      artifact::Filter,
      primitives::{EntityType, Id},
    },
    repo,
  },
  ui::{
    components::{ArtifactEntry, ArtifactListView},
    envelope::Envelope,
    json,
  },
};

/// List artifacts in the current project.
#[derive(Args, Debug)]
pub struct Command {
  /// Show all artifacts, including archived.
  #[arg(long, short)]
  all: bool,
  /// Show only archived artifacts.
  #[arg(long, visible_alias = "archived-only")]
  archived: bool,
  #[command(flatten)]
  limit: LimitArgs,
  /// Filter by tag.
  #[arg(long, short)]
  tag: Option<String>,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Query artifacts with the requested filters and render them as a table, JSON, or plain IDs.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact list: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let filter = Filter {
      all: self.all,
      only_archived: self.archived,
      tag: self.tag.clone(),
    };

    let mut artifacts = repo::artifact::all(&conn, project_id, &filter).await?;
    self.limit.apply(&mut artifacts);

    let id_shorts: Vec<String> = artifacts.iter().map(|a| a.id().short().to_string()).collect();

    if self.output.json {
      let pairs: Vec<(Id, _)> = artifacts.iter().map(|a| (a.id().clone(), a.clone())).collect();
      let envelopes = Envelope::load_many(&conn, EntityType::Artifact, &pairs, false).await?;
      let json = serde_json::to_string_pretty(&envelopes)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for id in &id_shorts {
        println!("{id}");
      }
      return Ok(());
    }

    let id_full_strs: Vec<String> = artifacts.iter().map(|a| a.id().to_string()).collect();
    let id_refs: Vec<&str> = id_full_strs.iter().map(String::as_str).collect();
    let prefix_lens = repo::artifact::prefix_lengths(&conn, project_id, &id_refs).await?;

    let artifact_ids: Vec<Id> = artifacts.iter().map(|a| a.id().clone()).collect();
    let tag_map = repo::tag::for_entities(&conn, EntityType::Artifact, &artifact_ids).await?;

    let mut entries = Vec::new();
    for (i, (artifact, id_short)) in artifacts.iter().zip(id_shorts.iter()).enumerate() {
      let tags: Vec<String> = tag_map
        .get(artifact.id())
        .map(|tags| tags.iter().map(|t| t.label().to_string()).collect())
        .unwrap_or_default();
      entries.push(ArtifactEntry {
        archived: artifact.is_archived(),
        id: id_short.clone(),
        prefix_len: prefix_lens[i],
        tags,
        title: artifact.title().to_string(),
      });
    }

    crate::io::pager::page(&format!("{}\n", ArtifactListView::new(entries)), context)?;

    Ok(())
  }
}
