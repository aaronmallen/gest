use clap::Args;

use crate::{
  config,
  config::Config,
  store,
  ui::{components::ArtifactDetail, theme::Theme},
};

/// Display an artifact's full details and body
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix
  pub id: String,
  /// Output artifact details as JSON
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  pub fn call(&self, config: &Config, _theme: &Theme) -> crate::Result<()> {
    log::info!("showing artifact with prefix '{}'", self.id);
    let data_dir = config::data_dir(config)?;
    log::debug!("resolving artifact ID from prefix '{}'", self.id);
    let id = store::resolve_artifact_id(&data_dir, &self.id, true)?;
    log::debug!("resolved artifact ID: {id}");
    let artifact = store::read_artifact(&data_dir, &id)?;
    log::trace!(
      "artifact '{}' loaded, outputting as {}",
      artifact.title,
      if self.json { "json" } else { "detail" }
    );

    if self.json {
      let json = serde_json::json!({
        "id": artifact.id.to_string(),
        "title": artifact.title,
        "type": artifact.kind,
        "tags": artifact.tags,
        "body": artifact.body,
        "metadata": format_metadata_for_json(&artifact.metadata),
        "created_at": artifact.created_at.to_rfc3339(),
        "updated_at": artifact.updated_at.to_rfc3339(),
        "archived_at": artifact.archived_at.map(|dt| dt.to_rfc3339()),
      });
      println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
      ArtifactDetail::new(&artifact).write_to(&mut std::io::stdout())?;
    }

    Ok(())
  }
}

fn format_metadata_for_json(mapping: &yaml_serde::Mapping) -> serde_json::Value {
  let mut map = serde_json::Map::new();
  for (k, v) in mapping {
    if let yaml_serde::Value::String(key) = k {
      map.insert(key.clone(), yaml_value_to_json(v));
    }
  }
  serde_json::Value::Object(map)
}

fn yaml_value_to_json(v: &yaml_serde::Value) -> serde_json::Value {
  match v {
    yaml_serde::Value::Null => serde_json::Value::Null,
    yaml_serde::Value::Bool(b) => serde_json::Value::Bool(*b),
    yaml_serde::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        serde_json::Value::Number(i.into())
      } else if let Some(f) = n.as_f64() {
        serde_json::json!(f)
      } else {
        serde_json::Value::Null
      }
    }
    yaml_serde::Value::String(s) => serde_json::Value::String(s.clone()),
    yaml_serde::Value::Sequence(seq) => serde_json::Value::Array(seq.iter().map(yaml_value_to_json).collect()),
    yaml_serde::Value::Mapping(m) => format_metadata_for_json(m),
    yaml_serde::Value::Tagged(t) => yaml_value_to_json(&t.value),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod format_metadata_for_json {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_string_values() {
      let mut mapping = yaml_serde::Mapping::new();
      mapping.insert(
        yaml_serde::Value::String("key".to_string()),
        yaml_serde::Value::String("value".to_string()),
      );
      let json = format_metadata_for_json(&mapping);
      assert_eq!(json, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn it_handles_empty_mapping() {
      let mapping = yaml_serde::Mapping::new();
      let json = format_metadata_for_json(&mapping);
      assert_eq!(json, serde_json::json!({}));
    }
  }
}
