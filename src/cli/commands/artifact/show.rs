use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::ArtifactDetail, json},
};

/// Show an artifact by ID or prefix.
#[derive(Args, Debug)]
pub struct Command {
  /// The artifact ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact show: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, "artifacts", &self.id).await?;
    let artifact = repo::artifact::find_by_id(&conn, id.clone())
      .await?
      .ok_or_else(|| Error::Resolve(repo::resolve::Error::NotFound(self.id.clone())))?;

    let short_id = artifact.id().short();
    if self.output.json || self.output.quiet {
      self.output.print_entity(&artifact, &short_id, String::new)?;
      return Ok(());
    }

    let prefix_len = if artifact.is_archived() {
      repo::artifact::shortest_all_prefix(&conn, project_id).await?
    } else {
      repo::artifact::shortest_active_prefix(&conn, project_id).await?
    };

    let tags = repo::tag::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;
    let notes = repo::note::for_entity(&conn, EntityType::Artifact, artifact.id()).await?;

    let mut view = ArtifactDetail::new(artifact.id().short(), artifact.title().to_string()).id_prefix_len(prefix_len);

    if artifact.is_archived() {
      view = view.archived();
    }
    if !tags.is_empty() {
      view = view.tags(tags);
    }
    if !artifact.body().is_empty() {
      view = view.body(artifact.body());
    }
    for note in &notes {
      view = view.note(note.id().short(), note.body());
    }

    print!("{view}");
    Ok(())
  }
}
