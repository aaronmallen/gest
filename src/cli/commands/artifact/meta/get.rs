use clap::Args;

use crate::{config, config::Config, store};

/// Get a metadata value from an artifact
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Dot-delimited key path (e.g. outer.inner)
  pub path: String,
}

impl Command {
  pub fn call(&self, config: &Config) -> crate::Result<()> {
    log::info!(
      "getting metadata '{}' from artifact with prefix '{}'",
      self.path,
      self.id
    );
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");
    let artifact = store::read_artifact(&data_dir, &id)?;

    log::debug!("resolving dot path '{}'", self.path);
    let value = resolve_dot_path(&yaml_serde::Value::Mapping(artifact.metadata), &self.path)
      .ok_or_else(|| crate::Error::generic(format!("Metadata key not found: '{}'", self.path)))?;

    log::trace!("metadata key '{}' resolved for artifact {id}", self.path);
    print_yaml_value(&value);
    Ok(())
  }
}

fn print_yaml_value(value: &yaml_serde::Value) {
  match value {
    yaml_serde::Value::String(s) => println!("{s}"),
    yaml_serde::Value::Bool(b) => println!("{b}"),
    yaml_serde::Value::Number(n) => println!("{n}"),
    yaml_serde::Value::Null => println!("null"),
    yaml_serde::Value::Sequence(seq) => {
      for item in seq {
        print!("- ");
        print_yaml_value(item);
      }
    }
    yaml_serde::Value::Mapping(_) | yaml_serde::Value::Tagged(_) => {
      if let Ok(s) = yaml_serde::to_string(value) {
        print!("{s}");
      }
    }
  }
}

fn resolve_dot_path(root: &yaml_serde::Value, path: &str) -> Option<yaml_serde::Value> {
  let segments: Vec<&str> = path.split('.').collect();
  let mut current = root.clone();

  for segment in &segments {
    match current {
      yaml_serde::Value::Mapping(m) => {
        current = m.get(yaml_serde::Value::String(segment.to_string()))?.clone();
      }
      _ => return None,
    }
  }

  Some(current)
}

#[cfg(test)]
mod tests {
  use super::*;

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
      let result = resolve_dot_path(&mapping_value(mapping), "outer.nested");
      assert_eq!(result, Some(yaml_serde::Value::String("deep".to_string())));
    }

    #[test]
    fn it_resolves_top_level_key() {
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("key".to_string()),
        yaml_serde::Value::String("value".to_string()),
      );
      let result = resolve_dot_path(&mapping_value(mapping), "key");
      assert_eq!(result, Some(yaml_serde::Value::String("value".to_string())));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
      let mapping = yaml_serde::Mapping::new();
      let result = resolve_dot_path(&mapping_value(mapping), "missing");
      assert_eq!(result, None);
    }

    #[test]
    fn it_returns_none_for_non_mapping_intermediate() {
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("key".to_string()),
        yaml_serde::Value::String("value".to_string()),
      );
      let result = resolve_dot_path(&mapping_value(mapping), "key.nested");
      assert_eq!(result, None);
    }

    fn mapping_value(mapping: yaml_serde::Mapping) -> yaml_serde::Value {
      yaml_serde::Value::Mapping(mapping)
    }
  }
}
