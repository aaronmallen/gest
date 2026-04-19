use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  store::{model::primitives::EntityType, repo},
  ui::{components::ArtifactDetail, envelope::Envelope, json},
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
  /// Resolve the artifact and render its details, tags, and notes.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("artifact show: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let conn = context.store().connect().await?;

    let id = repo::resolve::resolve_id(&conn, repo::resolve::Table::Artifacts, &self.id).await?;
    let artifact = repo::artifact::find_required_by_id(&conn, id.clone()).await?;

    let short_id = artifact.id().short();
    if self.output.json || self.output.quiet {
      let envelope = Envelope::load_one(&conn, EntityType::Artifact, artifact.id(), &artifact, true).await?;
      self.output.print_envelope(&envelope, &short_id, String::new)?;
      return Ok(());
    }

    let id_str = artifact.id().to_string();
    let prefix_lens = repo::artifact::prefix_lengths(&conn, project_id, &[id_str.as_str()]).await?;
    let prefix_len = prefix_lens[0];

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

    crate::io::pager::page(&format!("{view}"), context)?;
    Ok(())
  }
}
