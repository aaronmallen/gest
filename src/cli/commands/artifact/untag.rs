use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::TagChange, theme::Theme},
};

/// Remove tags from an artifact
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Tags to remove (space-separated)
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    log::info!("untagging artifact with prefix '{}'", self.id);
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");
    let mut artifact = store::read_artifact(&data_dir, &id)?;

    log::debug!("tags to remove: {:?}", self.tags);
    artifact.tags.retain(|t| !self.tags.contains(t));

    artifact.updated_at = Utc::now();
    store::write_artifact(&data_dir, &artifact)?;
    log::trace!("artifact {id} untagged successfully");
    TagChange::new("Untagged", "artifact", &id, &self.tags).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;
  use crate::test_helpers::{make_test_artifact, make_test_config};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_tags() {
      let (_dir, config) = setup();
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }

    #[test]
    fn it_can_remove_all_tags() {
      let (_dir, config) = setup();
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["rust".to_string(), "cli".to_string()];
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert!(loaded.tags.is_empty());
    }

    #[test]
    fn it_handles_nonexistent_tags_gracefully() {
      let (_dir, config) = setup();
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["rust".to_string()];
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string()]);
    }
  }

  fn setup() -> (TempDir, crate::config::Config) {
    let dir = TempDir::new().unwrap();
    let config = make_test_config(dir.path());
    (dir, config)
  }
}
