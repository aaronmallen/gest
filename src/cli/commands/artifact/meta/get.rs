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
    let layout = &ctx.layout;
    let id = store::resolve_artifact_id(layout, &self.id, false)?;
    let artifact = store::read_artifact(layout, &id)?;

    let root = yaml_serde::Value::Mapping(artifact.metadata);
    let value = resolve_dot_path(&root, &self.path)
      .ok_or_else(|| cli::Error::generic(format!("Metadata key not found: '{}'", self.path)))?;

    print_yaml_value(&value);
    Ok(())
  }
}

/// Print a YAML value to stdout, serializing complex types as JSON.
fn print_yaml_value(value: &yaml_serde::Value) {
  match value {
    yaml_serde::Value::String(s) => println!("{s}"),
    yaml_serde::Value::Bool(b) => println!("{b}"),
    yaml_serde::Value::Number(n) => println!("{n}"),
    yaml_serde::Value::Null => println!("null"),
    yaml_serde::Value::Sequence(seq) => {
      let json = serde_json::to_string_pretty(seq).unwrap_or_else(|_| format!("{seq:?}"));
      println!("{json}");
    }
    yaml_serde::Value::Mapping(m) => {
      let json = serde_json::to_string_pretty(m).unwrap_or_else(|_| format!("{m:?}"));
      println!("{json}");
    }
    yaml_serde::Value::Tagged(t) => {
      print_yaml_value(&t.value);
    }
  }
}

/// Walk a YAML mapping by splitting `path` on `.` and returning the leaf value.
fn resolve_dot_path(root: &yaml_serde::Value, path: &str) -> Option<yaml_serde::Value> {
  let segments: Vec<&str> = path.split('.').collect();
  let mut current = root.clone();

  for segment in &segments {
    match current {
      yaml_serde::Value::Mapping(m) => {
        let key = yaml_serde::Value::String(segment.to_string());
        current = m.get(&key)?.clone();
      }
      _ => return None,
    }
  }

  Some(current)
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
      store::write_artifact(&ctx.layout, &artifact).unwrap();

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
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
      };
      cmd.call(&ctx).unwrap();
    }
  }

  mod resolve_dot_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_resolves_nested_key() {
      let mut inner = yaml_serde::Mapping::new();
      inner.insert(
        yaml_serde::Value::String("nested".to_string()),
        yaml_serde::Value::String("deep".to_string()),
      );
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("outer".to_string()),
        yaml_serde::Value::Mapping(inner),
      );
      let result = resolve_dot_path(&yaml_serde::Value::Mapping(mapping), "outer.nested");
      assert_eq!(result, Some(yaml_serde::Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("key".to_string()),
        yaml_serde::Value::String("value".to_string()),
      );
      let result = resolve_dot_path(&yaml_serde::Value::Mapping(mapping), "key");
      assert_eq!(result, Some(yaml_serde::Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let mapping = yaml_serde::Mapping::new();
      let result = resolve_dot_path(&yaml_serde::Value::Mapping(mapping), "missing");
      assert_eq!(result, None);
    }
  }
}
