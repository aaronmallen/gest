use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Set a metadata value on an artifact using a dot-delimited key path.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Output as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Dot-delimited key path (e.g. `config.timeout`).
  pub path: String,
  /// Print only the entity ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
  /// Value to set (strings, numbers, booleans, and null are auto-detected).
  pub value: String,
}

impl Command {
  /// Resolve the artifact, set the metadata key to the given value, and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let id = store::resolve_artifact_id(config, &self.id, false)?;
    let mut artifact = store::read_artifact(config, &id)?;

    store::artifact_meta::set_dot_path(&mut artifact.metadata, &self.path, &self.value)?;

    artifact.updated_at = Utc::now();
    store::write_artifact(config, &artifact)?;

    if self.json {
      println!("{}", serde_json::to_string_pretty(&artifact)?);
    } else if self.quiet {
      println!("{}", artifact.id);
    } else {
      let msg = format!("Set {}.{} = {}", id, self.path, self.value);
      println!("{}", SuccessMessage::new(&msg, &ctx.theme));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::{make_test_artifact, make_test_context};

    #[test]
    fn it_sets_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
        path: "priority".to_string(),
        quiet: false,
        value: "high".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.settings, &artifact.id).unwrap();

      assert_eq!(
        loaded.metadata.get(yaml_serde::Value::String("priority".to_string())),
        Some(&yaml_serde::Value::String("high".to_string()))
      );
    }

    #[test]
    fn it_sets_nested_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
        path: "config.timeout".to_string(),
        quiet: false,
        value: "30".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.settings, &artifact.id).unwrap();
      let config_key = yaml_serde::Value::String("config".to_string());
      let config_val = loaded.metadata.get(config_key).unwrap();

      if let yaml_serde::Value::Mapping(m) = config_val {
        let timeout_key = yaml_serde::Value::String("timeout".to_string());
        assert_eq!(
          m.get(timeout_key),
          Some(&yaml_serde::Value::Number(yaml_serde::Number::from(30)))
        );
      } else {
        panic!("Expected mapping for config key");
      }
    }
  }
}
