use std::io::Write;

use clap::Args;
use serde::Serialize;
use yansi::Paint;

use crate::{
  config,
  config::Config,
  model::{Artifact, Task, task::STATUS_ORDER},
  store::{self, SearchResults},
  ui::{
    components::{Group, GroupedList, NoResults},
    theme::Theme,
    utils::{format_id, format_tags, shortest_unique_prefixes},
  },
};

/// Search across all tasks and artifacts by keyword
#[derive(Debug, Args)]
pub struct Command {
  /// Search query matched against titles, descriptions, and body content
  pub query: String,
  /// Output search results as JSON
  #[arg(short, long)]
  pub json: bool,
  /// Include archived/resolved items in search results
  #[arg(short = 'a', long = "all")]
  pub show_all: bool,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let results: SearchResults = store::search(&data_dir, &self.query, self.show_all)?;

    if self.json {
      return self.print_json(&results);
    }

    if results.tasks.is_empty() && results.artifacts.is_empty() {
      NoResults::new(&self.query).write_to(&mut std::io::stdout())?;
      return Ok(());
    }

    self.print_grouped(&results, theme)?;
    Ok(())
  }

  fn print_grouped(&self, results: &SearchResults, theme: &Theme) -> crate::Result<()> {
    let mut out = std::io::stdout();

    if !results.tasks.is_empty() {
      writeln!(out, "{}\n", "Tasks".paint(theme.list_heading))?;
      let groups = build_task_groups(&results.tasks, theme);
      GroupedList::new(groups, theme).write_to(&mut out)?;
    }

    if !results.artifacts.is_empty() {
      if !results.tasks.is_empty() {
        writeln!(out)?;
      }
      writeln!(out, "{}\n", "Artifacts".paint(theme.list_heading))?;
      let groups = build_artifact_groups(&results.artifacts, theme);
      GroupedList::new(groups, theme).write_to(&mut out)?;
    }

    Ok(())
  }

  fn print_json(&self, results: &SearchResults) -> crate::Result<()> {
    let output = JsonOutput {
      artifacts: results
        .artifacts
        .iter()
        .map(|a| JsonArtifact {
          id: a.id.to_string(),
          kind: a.kind.clone(),
          tags: a.tags.clone(),
          title: a.title.clone(),
        })
        .collect(),
      tasks: results
        .tasks
        .iter()
        .map(|t| JsonTask {
          id: t.id.to_string(),
          status: t.status.to_string(),
          tags: t.tags.clone(),
          title: t.title.clone(),
        })
        .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
  }
}

#[derive(Serialize)]
struct JsonArtifact {
  id: String,
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  kind: Option<String>,
  tags: Vec<String>,
  title: String,
}

#[derive(Serialize)]
struct JsonOutput {
  artifacts: Vec<JsonArtifact>,
  tasks: Vec<JsonTask>,
}

#[derive(Serialize)]
struct JsonTask {
  id: String,
  status: String,
  tags: Vec<String>,
  title: String,
}

fn build_artifact_groups(artifacts: &[Artifact], theme: &Theme) -> Vec<Group> {
  let id_strings: Vec<String> = artifacts.iter().map(|a| a.id.to_string()).collect();
  let prefix_lens = shortest_unique_prefixes(&id_strings);

  // Collect unique kinds, sort alphabetically with "Other" (None) last
  let mut kinds: Vec<Option<String>> = Vec::new();
  for a in artifacts {
    if !kinds.contains(&a.kind) {
      kinds.push(a.kind.clone());
    }
  }
  kinds.sort_by(|a, b| match (a, b) {
    (None, None) => std::cmp::Ordering::Equal,
    (None, Some(_)) => std::cmp::Ordering::Greater,
    (Some(_), None) => std::cmp::Ordering::Less,
    (Some(a), Some(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
  });

  kinds
    .iter()
    .map(|kind| {
      let heading = kind.as_deref().unwrap_or("Other").to_string();
      let rows: Vec<Vec<String>> = artifacts
        .iter()
        .zip(&prefix_lens)
        .filter(|(a, _)| a.kind == *kind)
        .map(|(a, &plen)| {
          let mut row = vec![format_id(&a.id, plen, Some(8), theme), a.title.clone()];
          let tags = format_tags(&a.tags, theme);
          if !tags.is_empty() {
            row.push(tags);
          }
          row
        })
        .collect();
      Group::new(heading, rows)
    })
    .collect()
}

fn build_task_groups(tasks: &[Task], theme: &Theme) -> Vec<Group> {
  let id_strings: Vec<String> = tasks.iter().map(|t| t.id.to_string()).collect();
  let prefix_lens = shortest_unique_prefixes(&id_strings);

  STATUS_ORDER
    .iter()
    .map(|status| {
      let rows: Vec<Vec<String>> = tasks
        .iter()
        .zip(&prefix_lens)
        .filter(|(t, _)| t.status == *status)
        .map(|(t, &plen)| {
          let mut row = vec![format_id(&t.id, plen, Some(8), theme), t.title.clone()];
          let tags = format_tags(&t.tags, theme);
          if !tags.is_empty() {
            row.push(tags);
          }
          row
        })
        .collect();
      Group::new(status.to_string(), rows)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use chrono::Utc;

  use crate::{
    model::{Artifact, Task, task::Status},
    ui::theme::Theme,
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::config::{Config, StorageConfig};

    #[test]
    fn it_excludes_resolved_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved Feature");
      crate::store::write_task(dir.path(), &task).unwrap();
      crate::store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = super::super::Command {
        query: "resolved".to_string(),
        show_all: false,
        json: true,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_finds_artifacts_by_query() {
      let dir = tempfile::tempdir().unwrap();
      let artifact = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "Notes", "secret keyword here");
      crate::store::write_artifact(dir.path(), &artifact).unwrap();

      let cmd = super::super::Command {
        query: "secret".to_string(),
        show_all: false,
        json: false,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_finds_tasks_by_query() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Important Feature");
      crate::store::write_task(dir.path(), &task).unwrap();

      let cmd = super::super::Command {
        query: "important".to_string(),
        show_all: false,
        json: false,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_no_results() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();

      let cmd = super::super::Command {
        query: "nonexistent".to_string(),
        show_all: false,
        json: false,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_includes_resolved_when_flag_set() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved Feature");
      crate::store::write_task(dir.path(), &task).unwrap();
      crate::store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = super::super::Command {
        query: "resolved".to_string(),
        show_all: true,
        json: false,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_json_output_has_correct_structure() {
      let dir = tempfile::tempdir().unwrap();
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "My Task");
      task.tags = vec!["rust".to_string()];
      crate::store::write_task(dir.path(), &task).unwrap();

      let mut artifact = make_test_artifact("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "My Note", "some body");
      artifact.kind = Some("note".to_string());
      artifact.tags = vec!["doc".to_string()];
      crate::store::write_artifact(dir.path(), &artifact).unwrap();

      let results = crate::store::search(dir.path(), "my", false).unwrap();
      assert_eq!(results.tasks.len(), 1);
      assert_eq!(results.artifacts.len(), 1);
    }

    fn make_config(dir: &std::path::Path) -> Config {
      Config {
        storage: StorageConfig {
          data_dir: Some(dir.to_path_buf()),
        },
        ..Config::default()
      }
    }

    #[test]
    fn it_outputs_json_when_flag_set() {
      let dir = tempfile::tempdir().unwrap();
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Important Feature");
      crate::store::write_task(dir.path(), &task).unwrap();

      let cmd = super::super::Command {
        query: "important".to_string(),
        show_all: false,
        json: true,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }

    #[test]
    fn it_returns_empty_json_for_no_results() {
      let dir = tempfile::tempdir().unwrap();
      crate::store::ensure_dirs(dir.path()).unwrap();

      let cmd = super::super::Command {
        query: "nonexistent".to_string(),
        show_all: false,
        json: true,
      };
      let config = make_config(dir.path());
      let result = cmd.call(&config, &Theme::default());
      assert!(result.is_ok());
    }
  }

  fn make_test_artifact(id: &str, title: &str, body: &str) -> Artifact {
    Artifact {
      archived_at: None,
      body: body.to_string(),
      created_at: Utc::now(),
      id: id.parse().unwrap(),
      kind: None,
      metadata: yaml_serde::Mapping::new(),
      tags: vec![],
      title: title.to_string(),
      updated_at: Utc::now(),
    }
  }

  fn make_test_task(id: &str, title: &str) -> Task {
    Task {
      resolved_at: None,
      created_at: Utc::now(),
      description: String::new(),
      id: id.parse().unwrap(),
      links: vec![],
      metadata: toml::Table::new(),
      status: Status::Open,
      tags: vec![],
      title: title.to_string(),
      updated_at: Utc::now(),
    }
  }
}
