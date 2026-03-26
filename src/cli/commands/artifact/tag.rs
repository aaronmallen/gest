use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::TagChange, theme::Theme},
};

/// Add tags to an artifact
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Tags to add (space-separated)
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    let mut artifact = store::read_artifact(&data_dir, &id)?;

    for tag in &self.tags {
      if !artifact.tags.contains(tag) {
        artifact.tags.push(tag.clone());
      }
    }

    artifact.updated_at = Utc::now();
    store::write_artifact(&data_dir, &artifact)?;
    TagChange::new("Tagged", "artifact", &id, &self.tags).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}
