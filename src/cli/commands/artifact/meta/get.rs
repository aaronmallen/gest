use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
};

/// Get a metadata value from an artifact using a dot-delimited key path.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
}

impl Command {
  /// Resolve the artifact, look up the metadata key, and print its value.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let id = store::resolve_artifact_id(config, &self.id, false)?;
    let artifact = store::read_artifact(config, &id)?;

    let root = yaml_serde::Value::Mapping(artifact.metadata);
    let value = store::artifact_meta::resolve_dot_path(&root, &self.path)
      .ok_or_else(|| cli::Error::generic(format!("Metadata key not found: '{}'", self.path)))?;

    store::artifact_meta::print_yaml_value(value);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use super::*;
    use crate::test_helpers::{make_test_artifact, make_test_context};

    #[test]
    fn it_errors_on_missing_path() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "nonexistent".to_string(),
      };
      let result = cmd.call(&ctx);
      assert!(result.is_err());
    }

    #[test]
    fn it_reads_metadata_value() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.metadata.insert(
        yaml_serde::Value::String("priority".to_string()),
        yaml_serde::Value::String("high".to_string()),
      );
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
      };
      cmd.call(&ctx).unwrap();
    }
  }
}
