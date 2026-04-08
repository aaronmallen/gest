use clap::Args;

use crate::{
  AppContext,
  cli::{Error, limit::LimitArgs},
  store::{
    model::{artifact::Filter, primitives::EntityType},
    repo,
  },
  ui::{
    components::{ArtifactEntry, ArtifactListView},
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
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
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
      let json = serde_json::to_string_pretty(&artifacts)?;
      println!("{json}");
      return Ok(());
    }

    if self.output.quiet {
      for id in &id_shorts {
        println!("{id}");
      }
      return Ok(());
    }

    let prefix_len = if self.all || self.archived {
      repo::artifact::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::artifact::shortest_active_prefix(&conn, project_id).await?
    };

    let mut entries = Vec::new();
    for (artifact, id_short) in artifacts.iter().zip(id_shorts.iter()) {
      let tags = repo::tag::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;
      entries.push(ArtifactEntry {
        archived: artifact.is_archived(),
        id: id_short.clone(),
        tags,
        title: artifact.title().to_string(),
      });
    }

    crate::io::pager::page(&format!("{}\n", ArtifactListView::new(entries, prefix_len)), context)?;

    Ok(())
  }
}
