use chrono::Utc;
use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::Confirmation, theme::Theme},
};

/// Set a metadata value on an artifact
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Dot-delimited key path (e.g. outer.inner)
  pub path: String,
  /// Value to set (strings, numbers, booleans, and null are auto-detected)
  pub value: String,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    log::info!("setting metadata '{}' on artifact with prefix '{}'", self.path, self.id);
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");
    let mut artifact = store::read_artifact(&data_dir, &id)?;

    log::debug!("setting dot path '{}' to '{}'", self.path, self.value);
    set_dot_path(&mut artifact.metadata, &self.path, &self.value)?;

    artifact.updated_at = Utc::now();
    store::write_artifact(&data_dir, &artifact)?;
    log::trace!("metadata '{}' set on artifact {id}", self.path);
    Confirmation::new("Updated", "artifact", &artifact.id).write_to(&mut std::io::stdout(), theme)?;
    Ok(())
  }
}

fn parse_yaml_value(s: &str) -> yaml_serde::Value {
  match s {
    "true" => yaml_serde::Value::Bool(true),
    "false" => yaml_serde::Value::Bool(false),
    "null" => yaml_serde::Value::Null,
    _ => {
      if let Ok(n) = s.parse::<i64>() {
        yaml_serde::Value::Number(n.into())
      } else if let Ok(n) = s.parse::<f64>() {
        yaml_serde::Value::Number(yaml_serde::Number::from(n))
      } else {
        yaml_serde::Value::String(s.to_string())
      }
    }
  }
}

fn set_dot_path(mapping: &mut yaml_serde::Mapping, path: &str, value: &str) -> crate::Result<()> {
  let segments: Vec<&str> = path.split('.').collect();
  let yaml_value = parse_yaml_value(value);

  if segments.len() == 1 {
    mapping.insert(yaml_serde::Value::String(segments[0].to_string()), yaml_value);
    return Ok(());
  }

  set_nested(mapping, &segments, yaml_value);
  Ok(())
}

fn set_nested(mapping: &mut yaml_serde::Mapping, segments: &[&str], value: yaml_serde::Value) {
  let key = yaml_serde::Value::String(segments[0].to_string());

  if segments.len() == 1 {
    mapping.insert(key, value);
    return;
  }

  let mut nested = match mapping.remove(&key) {
    Some(yaml_serde::Value::Mapping(m)) => m,
    _ => yaml_serde::Mapping::new(),
  };

  set_nested(&mut nested, &segments[1..], value);
  mapping.insert(key, yaml_serde::Value::Mapping(nested));
}

#[cfg(test)]
mod tests {
  use super::*;

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
      assert_eq!(parse_yaml_value("42"), yaml_serde::Value::Number(42.into()));
    }

    #[test]
    fn it_parses_null() {
      assert_eq!(parse_yaml_value("null"), yaml_serde::Value::Null);
    }
  }

  mod set_dot_path {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_overwrites_existing_value() {
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("key".to_string()),
        yaml_serde::Value::String("old".to_string()),
      );
      set_dot_path(&mut mapping, "key", "new").unwrap();
      assert_eq!(
        mapping.get(&yaml_serde::Value::String("key".to_string())),
        Some(&yaml_serde::Value::String("new".to_string()))
      );
    }

    #[test]
    fn it_preserves_sibling_keys_in_nested_path() {
      let mut mapping = yaml_serde::Mapping::new();
      set_dot_path(&mut mapping, "outer.first", "one").unwrap();
      set_dot_path(&mut mapping, "outer.second", "two").unwrap();

      let outer = mapping.get(&yaml_serde::Value::String("outer".to_string())).unwrap();
      if let yaml_serde::Value::Mapping(inner) = outer {
        assert_eq!(
          inner.get(&yaml_serde::Value::String("first".to_string())),
          Some(&yaml_serde::Value::String("one".to_string()))
        );
        assert_eq!(
          inner.get(&yaml_serde::Value::String("second".to_string())),
          Some(&yaml_serde::Value::String("two".to_string()))
        );
      } else {
        panic!("Expected mapping at 'outer'");
      }
    }

    #[test]
    fn it_sets_nested_key() {
      let mut mapping = yaml_serde::Mapping::new();
      set_dot_path(&mut mapping, "outer.inner", "deep").unwrap();

      let outer = mapping.get(&yaml_serde::Value::String("outer".to_string())).unwrap();
      if let yaml_serde::Value::Mapping(inner) = outer {
        assert_eq!(
          inner.get(&yaml_serde::Value::String("inner".to_string())),
          Some(&yaml_serde::Value::String("deep".to_string()))
        );
      } else {
        panic!("Expected mapping at 'outer'");
      }
    }

    #[test]
    fn it_sets_top_level_key() {
      let mut mapping = yaml_serde::Mapping::new();
      set_dot_path(&mut mapping, "key", "value").unwrap();
      assert_eq!(
        mapping.get(&yaml_serde::Value::String("key".to_string())),
        Some(&yaml_serde::Value::String("value".to_string()))
      );
    }
  }
}
