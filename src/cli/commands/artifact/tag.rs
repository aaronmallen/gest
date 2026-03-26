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
    log::info!("tagging artifact with prefix '{}'", self.id);
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");
    let mut artifact = store::read_artifact(&data_dir, &id)?;

    for tag in &self.tags {
      if !artifact.tags.contains(tag) {
        artifact.tags.push(tag.clone());
      }
    }
    log::debug!("tags to add: {:?}", self.tags);

    artifact.updated_at = Utc::now();
    store::write_artifact(&data_dir, &artifact)?;
    log::trace!("artifact {id} tagged successfully");
    TagChange::new("Tagged", "artifact", &id, &self.tags).write_to(&mut std::io::stdout())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use chrono::Utc;
  use tempfile::TempDir;

  use super::*;
  use crate::model::Artifact;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_tags() {
      let (_dir, config) = setup();
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_deduplicates_tags() {
      let (_dir, config) = setup();
      let mut artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["rust".to_string()];
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_preserves_existing_tags() {
      let (_dir, config) = setup();
      let mut artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["existing".to_string()];
      store::write_artifact(_dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["new".to_string()],
      };
      cmd.call(&config, &Theme::default()).unwrap();

      let loaded = store::read_artifact(_dir.path(), &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["existing".to_string(), "new".to_string()]);
    }
  }

  fn make_artifact(id: &str) -> Artifact {
    Artifact {
      archived_at: None,
      body: String::new(),
      created_at: Utc::now(),
      id: id.parse().unwrap(),
      kind: None,
      metadata: yaml_serde::Mapping::new(),
      tags: vec![],
      title: format!("Artifact {id}"),
      updated_at: Utc::now(),
    }
  }

  fn setup() -> (TempDir, crate::config::Config) {
    let dir = TempDir::new().unwrap();
    let config = crate::config::Config {
      storage: crate::config::StorageConfig {
        data_dir: Some(dir.path().to_path_buf()),
      },
      ..Default::default()
    };
    store::ensure_dirs(dir.path()).unwrap();
    (dir, config)
  }
}
