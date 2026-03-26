use std::collections::BTreeMap;

use clap::Args;

use crate::{
  config,
  config::Config,
  model::ArtifactFilter,
  store,
  ui::{
    components::{EmptyList, Group, GroupedList},
    theme::Theme,
    utils::{format_id, format_tags, shortest_unique_prefixes},
  },
};

/// List artifacts, optionally filtered by type or tag
#[derive(Debug, Args)]
pub struct Command {
  /// Show only archived artifacts
  #[arg(long, conflicts_with = "include_archived")]
  pub archived: bool,
  /// Include archived artifacts alongside active ones
  #[arg(short = 'a', long, conflicts_with = "archived")]
  pub include_archived: bool,
  /// Output artifact list as JSON
  #[arg(short, long)]
  pub json: bool,
  /// Filter by artifact type (e.g. spec, adr, rfc)
  #[arg(long = "type")]
  pub kind: Option<String>,
  /// Filter by tag
  #[arg(long)]
  pub tag: Option<String>,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    log::info!("listing artifacts");
    let filter = ArtifactFilter {
      include_archived: self.include_archived,
      only_archived: self.archived,
      kind: self.kind.clone(),
      tag: self.tag.clone(),
    };
    log::debug!(
      "filter: kind={:?}, tag={:?}, archived={}, include_archived={}",
      filter.kind,
      filter.tag,
      self.archived,
      self.include_archived
    );

    let data_dir = config::data_dir(config)?;
    let artifacts = store::list_artifacts(&data_dir, &filter)?;
    log::trace!("found {} artifact(s)", artifacts.len());

    if self.json {
      let json: Vec<serde_json::Value> = artifacts
        .iter()
        .map(|a| {
          serde_json::json!({
            "id": a.id.to_string(),
            "title": a.title,
            "type": a.kind,
            "tags": a.tags,
          })
        })
        .collect();
      println!("{}", serde_json::to_string_pretty(&json)?);
    } else if artifacts.is_empty() {
      EmptyList::new("artifacts").write_to(&mut std::io::stdout())?;
    } else {
      let id_strings: Vec<String> = artifacts.iter().map(|a| a.id.to_string()).collect();
      let prefix_lens = shortest_unique_prefixes(&id_strings);

      // Group artifacts by kind, sorted alphabetically with "Other" last
      let mut grouped: BTreeMap<String, Vec<(usize, &crate::model::Artifact)>> = BTreeMap::new();
      for (i, artifact) in artifacts.iter().enumerate() {
        let key = artifact.kind.clone().unwrap_or_default();
        grouped.entry(key).or_default().push((i, artifact));
      }

      // Sort each group by created_at (oldest first)
      for entries in grouped.values_mut() {
        entries.sort_by_key(|(_, a)| a.created_at);
      }

      // Build groups: named kinds alphabetically, then "Other" (empty kind) last
      let mut groups: Vec<Group> = Vec::new();
      let other_entries = grouped.remove("");

      for (kind, entries) in &grouped {
        let heading = capitalize(kind);
        let rows = entries
          .iter()
          .map(|(idx, a)| build_row(a, prefix_lens[*idx], theme))
          .collect();
        groups.push(Group::new(heading, rows));
      }

      if let Some(entries) = other_entries {
        let rows = entries
          .iter()
          .map(|(idx, a)| build_row(a, prefix_lens[*idx], theme))
          .collect();
        groups.push(Group::new("Other", rows));
      }

      GroupedList::new(groups, theme).write_to(&mut std::io::stdout())?;
    }

    Ok(())
  }
}

fn build_row(artifact: &crate::model::Artifact, prefix_len: usize, theme: &Theme) -> Vec<String> {
  vec![
    format_id(&artifact.id, prefix_len, Some(8), theme),
    artifact.title.clone(),
    format_tags(&artifact.tags, theme),
  ]
}

fn capitalize(s: &str) -> String {
  let mut chars = s.chars();
  match chars.next() {
    None => String::new(),
    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
  }
}

#[cfg(test)]
mod tests {
  use chrono::Utc;

  use super::*;
  use crate::{
    config::{Config, StorageConfig},
    model::Artifact,
    store,
  };

  mod call {
    use super::*;

    #[test]
    fn it_handles_empty_list() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: false,
        kind: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_lists_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      store::write_artifact(
        dir.path(),
        &make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Artifact One"),
      )
      .unwrap();

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: false,
        kind: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_outputs_json_empty() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: true,
        kind: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      store::write_artifact(
        dir.path(),
        &make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "JSON Artifact"),
      )
      .unwrap();

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: true,
        kind: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_filters_by_type() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let mut artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Typed");
      artifact.kind = Some("spec".to_string());
      store::write_artifact(dir.path(), &artifact).unwrap();

      let mut other = make_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Other");
      other.kind = Some("note".to_string());
      store::write_artifact(dir.path(), &other).unwrap();

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: false,
        kind: Some("spec".to_string()),
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_filters_by_tag() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let mut artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged");
      artifact.tags = vec!["important".to_string()];
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: false,
        kind: None,
        tag: Some("important".to_string()),
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_handles_filtered_empty() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let mut artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "A Spec");
      artifact.kind = Some("spec".to_string());
      store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = Command {
        archived: false,
        include_archived: false,
        json: false,
        kind: Some("nonexistent".to_string()),
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_includes_archived_artifacts() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_config(dir.path());
      let artifact = make_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Archived");
      store::write_artifact(dir.path(), &artifact).unwrap();
      store::archive_artifact(dir.path(), &artifact.id).unwrap();

      let cmd = Command {
        archived: false,
        include_archived: true,
        json: false,
        kind: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }
  }

  fn make_config(dir: &std::path::Path) -> Config {
    store::ensure_dirs(dir).unwrap();
    Config {
      storage: StorageConfig {
        data_dir: Some(dir.to_path_buf()),
      },
      ..Config::default()
    }
  }

  fn make_artifact(id: &str, title: &str) -> Artifact {
    let now = Utc::now();
    Artifact {
      archived_at: None,
      body: String::new(),
      created_at: now,
      id: id.parse().unwrap(),
      kind: None,
      metadata: yaml_serde::Mapping::new(),
      tags: vec![],
      title: title.to_string(),
      updated_at: now,
    }
  }
}
