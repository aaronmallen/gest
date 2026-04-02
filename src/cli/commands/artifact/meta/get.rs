use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::views::meta::MetaValueView,
};

/// Get a metadata value from an artifact using a dot-delimited key path.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Output as a JSON object.
  #[arg(long, conflicts_with = "raw")]
  pub json: bool,
  /// Dot-delimited key path (e.g. `outer.inner`).
  pub path: String,
  /// Output the bare value with no styling.
  #[arg(long, conflicts_with = "json")]
  pub raw: bool,
}

impl Command {
  /// Resolve the artifact, look up the metadata key, and print its value.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let id = store::resolve_artifact_id(config, &self.id, false)?;
    let artifact = store::read_artifact(config, &id)?;

    let root = yaml_serde::Value::Mapping(artifact.metadata);
    let value = store::artifact_meta::resolve_dot_path(&root, &self.path)
      .ok_or_else(|| cli::Error::NotFound(format!("Metadata key not found: '{}'", self.path)))?;

    let formatted = store::artifact_meta::format_yaml_value(value);
    let trimmed = formatted.trim_end_matches('\n');

    if self.json {
      let obj = serde_json::json!({ &self.path: trimmed });
      println!("{}", serde_json::to_string_pretty(&obj).unwrap());
    } else if self.raw {
      println!("{trimmed}");
    } else {
      println!("{}", MetaValueView::new(formatted, ctx.theme.artifact_detail_value));
    }

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
        json: false,
        path: "nonexistent".to_string(),
        raw: false,
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
        json: false,
        path: "priority".to_string(),
        raw: false,
      };
      cmd.call(&ctx).unwrap();
    }
  }
}
