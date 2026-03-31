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
  /// Dot-delimited key path (e.g. `config.timeout`).
  pub path: String,
  /// Value to set (strings, numbers, booleans, and null are auto-detected).
  pub value: String,
}

impl Command {
  /// Resolve the artifact, set the metadata key to the given value, and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_artifact_id(layout, &self.id, false)?;
    let mut artifact = store::read_artifact(layout, &id)?;

    set_dot_path(&mut artifact.metadata, &self.path, &self.value)?;

    artifact.updated_at = Utc::now();
    store::write_artifact(layout, &artifact)?;

    let msg = format!("Set {}.{} = {}", id, self.path, self.value);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

/// Parse a string into a typed YAML value (integer, float, bool, null, or string).
fn parse_yaml_value(s: &str) -> yaml_serde::Value {
  if let Ok(n) = s.parse::<i64>() {
    return yaml_serde::Value::Number(yaml_serde::Number::from(n));
  }
  if let Ok(n) = s.parse::<f64>() {
    return yaml_serde::Value::Number(yaml_serde::Number::from(n));
  }
  match s {
    "true" => yaml_serde::Value::Bool(true),
    "false" => yaml_serde::Value::Bool(false),
    "null" => yaml_serde::Value::Null,
    _ => yaml_serde::Value::String(s.to_string()),
  }
}

/// Set a value in a YAML mapping at the given dot-delimited path, creating intermediate mappings as needed.
fn set_dot_path(mapping: &mut yaml_serde::Mapping, path: &str, value: &str) -> cli::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();
  let yaml_value = parse_yaml_value(value);

  if segments.len() == 1 {
    mapping.insert(yaml_serde::Value::String(segments[0].to_string()), yaml_value);
    return Ok(());
  }

  set_nested(mapping, &segments, yaml_value);
  Ok(())
}

/// Recursively insert a value into nested YAML mappings along the given path segments.
fn set_nested(mapping: &mut yaml_serde::Mapping, segments: &[&str], value: yaml_serde::Value) {
  let key = yaml_serde::Value::String(segments[0].to_string());

  if segments.len() == 1 {
    mapping.insert(key, value);
    return;
  }

  let nested = mapping
    .entry(key.clone())
    .or_insert_with(|| yaml_serde::Value::Mapping(yaml_serde::Mapping::new()));

  if let yaml_serde::Value::Mapping(m) = nested {
    set_nested(m, &segments[1..], value);
  } else {
    let mut new_mapping = yaml_serde::Mapping::new();
    set_nested(&mut new_mapping, &segments[1..], value);
    mapping.insert(key, yaml_serde::Value::Mapping(new_mapping));
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
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "priority".to_string(),
        value: "high".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
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
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        path: "config.timeout".to_string(),
        value: "30".to_string(),
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
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

  mod parse_yaml_value {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_string() {
      assert_eq!(
        parse_yaml_value("hello"),
        yaml_serde::Value::String("hello".to_string())
      );
    }

    #[test]
    fn it_parses_booleans() {
      assert_eq!(parse_yaml_value("true"), yaml_serde::Value::Bool(true));
      assert_eq!(parse_yaml_value("false"), yaml_serde::Value::Bool(false));
    }

    #[test]
    fn it_parses_integers() {
      assert_eq!(
        parse_yaml_value("42"),
        yaml_serde::Value::Number(yaml_serde::Number::from(42))
      );
    }
  }
}
