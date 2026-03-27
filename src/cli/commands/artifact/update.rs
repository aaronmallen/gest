use std::io::IsTerminal;

use clap::Args;

use crate::{
  config,
  config::Config,
  model::ArtifactPatch,
  store,
  ui::{components::Confirmation, theme::Theme},
};

/// Update an artifact's title, body, type, tags, or metadata
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Replace the body content (opens $EDITOR if omitted and stdin is a terminal)
  #[arg(short, long)]
  pub body: Option<String>,
  /// Artifact type (e.g. spec, adr, rfc, note)
  #[arg(long = "type")]
  pub kind: Option<String>,
  /// Key=value metadata pair, merged with existing (repeatable, e.g. -m key=value)
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Replace all tags with this comma-separated list
  #[arg(long)]
  pub tags: Option<String>,
  /// New title
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    log::info!("updating artifact with prefix '{}'", self.id);
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");

    if self.body.is_none() && std::io::stdin().is_terminal() && crate::cli::editor::resolve_editor().is_some() {
      let path = store::artifact_path(&data_dir, &id);
      crate::cli::editor::open_editor(&path)?;
    }

    let metadata = if self.metadata.is_empty() {
      None
    } else {
      let pairs = crate::cli::helpers::split_key_value_pairs(&self.metadata)?;
      let mut map = store::read_artifact(&data_dir, &id)?.metadata;
      for (key, value) in pairs {
        map.insert(yaml_serde::Value::String(key), yaml_serde::Value::String(value));
      }
      Some(map)
    };

    let patch = ArtifactPatch {
      body: self.body.clone(),
      kind: self.kind.clone(),
      metadata,
      tags: self.tags.as_deref().map(crate::cli::helpers::parse_tags),
      title: self.title.clone(),
    };

    let artifact = store::update_artifact(&data_dir, &id, patch)?;
    log::trace!("artifact {} updated successfully", artifact.id);
    Confirmation::new("Updated", "artifact", &artifact.id).write_to(&mut std::io::stdout(), theme)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::Artifact,
    store,
    test_helpers::{make_test_artifact, make_test_config},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_updates_title_only() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        title: Some("New Title".to_string()),
        body: None,
        kind: None,
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_artifact(dir.path(), &artifact.id).unwrap();
      assert_eq!(updated.title, "New Title");
      assert_eq!(updated.body, "Original body");
      assert_eq!(updated.tags, vec!["original"]);
      assert_eq!(updated.kind, Some("note".to_string()));
    }

    #[test]
    fn it_updates_tags() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        title: None,
        body: None,
        kind: None,
        tags: Some("rust, cli".to_string()),
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_artifact(dir.path(), &artifact.id).unwrap();
      assert_eq!(updated.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_adds_metadata_entries() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        title: None,
        body: None,
        kind: None,
        tags: None,
        metadata: vec!["team=backend".to_string()],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_artifact(dir.path(), &artifact.id).unwrap();
      let priority = updated
        .metadata
        .get(yaml_serde::Value::String("priority".to_string()))
        .and_then(|v| v.as_str())
        .unwrap();
      assert_eq!(priority, "low");
      let team = updated
        .metadata
        .get(yaml_serde::Value::String("team".to_string()))
        .and_then(|v| v.as_str())
        .unwrap();
      assert_eq!(team, "backend");
    }

    #[test]
    fn it_sets_updated_at() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let original_updated = artifact.updated_at;
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        title: Some("Changed".to_string()),
        body: None,
        kind: None,
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_artifact(dir.path(), &artifact.id).unwrap();
      assert!(updated.updated_at >= original_updated);
    }

    #[test]
    fn it_updates_kind() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        title: None,
        body: None,
        kind: Some("adr".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_artifact(dir.path(), &artifact.id).unwrap();
      assert_eq!(updated.kind, Some("adr".to_string()));
    }
  }

  fn make_artifact(id: &str) -> Artifact {
    let mut metadata = yaml_serde::Mapping::new();
    metadata.insert(
      yaml_serde::Value::String("priority".to_string()),
      yaml_serde::Value::String("low".to_string()),
    );
    Artifact {
      body: "Original body".to_string(),
      kind: Some("note".to_string()),
      metadata,
      tags: vec!["original".to_string()],
      title: "Original Title".to_string(),
      ..make_test_artifact(id)
    }
  }
}
