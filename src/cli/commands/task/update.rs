use std::io::IsTerminal;

use clap::Args;

use crate::{
  config,
  config::Config,
  model::{Status, TaskPatch},
  store,
  ui::{components::Confirmation, theme::Theme},
};

/// Update a task's title, description, status, tags, or metadata
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix
  pub id: String,
  /// Actor assigned to this task
  #[arg(long)]
  pub assigned_to: Option<String>,
  /// New description (opens $EDITOR with current description if omitted and stdin is a terminal)
  #[arg(short, long)]
  pub description: Option<String>,
  /// Key=value metadata pair, merged with existing (repeatable, e.g. -m key=value)
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Execution phase for parallel grouping
  #[arg(long)]
  pub phase: Option<u16>,
  /// Priority level (0-4, where 0 is highest)
  #[arg(short, long)]
  pub priority: Option<u8>,
  /// New status: open, in-progress, done, or cancelled (done/cancelled auto-archives; open/in-progress unarchives)
  #[arg(short, long)]
  pub status: Option<String>,
  /// Replace all tags with this comma-separated list
  #[arg(long)]
  pub tags: Option<String>,
  /// New title
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    let data_dir = config::data_dir(config)?;
    let id = store::resolve_task_id(&data_dir, &self.id, true)?;

    let description =
      if self.description.is_none() && std::io::stdin().is_terminal() && crate::cli::editor::resolve_editor().is_some()
      {
        let task = store::read_task(&data_dir, &id)?;
        let content = crate::cli::editor::edit_temp(Some(&task.description), ".md")?;
        Some(content)
      } else {
        self.description.clone()
      };

    let status = self
      .status
      .as_deref()
      .map(|s| s.parse::<Status>().map_err(crate::Error::generic))
      .transpose()?;

    let metadata = if self.metadata.is_empty() {
      None
    } else {
      let pairs = crate::cli::helpers::split_key_value_pairs(&self.metadata)?;
      let mut table = store::read_task(&data_dir, &id)?.metadata;
      for (key, value) in pairs {
        table.insert(key, toml::Value::String(value));
      }
      Some(table)
    };

    let tags = self.tags.as_deref().map(crate::cli::helpers::parse_tags);

    let patch = TaskPatch {
      assigned_to: self.assigned_to.as_ref().map(|v| Some(v.clone())),
      description,
      metadata,
      phase: self.phase.map(Some),
      priority: self.priority.map(Some),
      status,
      tags,
      title: self.title.clone(),
    };

    let task = store::update_task(&data_dir, &id, patch)?;
    Confirmation::new("Updated", "task", &task.id).write_to(&mut std::io::stdout(), theme)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::{Link, RelationshipType},
    store,
    test_helpers::{make_test_config, make_test_task},
  };

  /// Build the specific "rich" task that update tests need (with description,
  /// links, metadata, tags).
  fn make_rich_task(id: &str) -> crate::model::Task {
    let mut task = make_test_task(id);
    task.description = "Original description".to_string();
    task.links = vec![Link {
      ref_: "https://example.com".to_string(),
      rel: RelationshipType::RelatesTo,
    }];
    task.metadata = {
      let mut table = toml::Table::new();
      table.insert("priority".to_string(), toml::Value::String("low".to_string()));
      table
    };
    task.tags = vec!["original".to_string()];
    task.title = "Original Title".to_string();
    task
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_metadata_entries() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: None,
        tags: None,
        metadata: vec!["team=backend".to_string()],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.metadata.get("priority").unwrap().as_str().unwrap(), "low");
      assert_eq!(updated.metadata.get("team").unwrap().as_str().unwrap(), "backend");
    }

    #[test]
    fn it_preserves_links_and_metadata() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: Some("New desc".to_string()),
        phase: None,
        priority: None,
        status: None,
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.links.len(), 1);
      assert_eq!(updated.links[0].rel, crate::model::RelationshipType::RelatesTo);
      assert_eq!(updated.metadata.get("priority").unwrap().as_str().unwrap(), "low");
    }

    #[test]
    fn it_resolves_task_on_terminal_status_cancelled() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: Some("cancelled".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.status, Status::Cancelled);
      assert!(updated.resolved_at.is_some());
    }

    #[test]
    fn it_resolves_task_on_terminal_status_done() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: Some("done".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      assert!(!dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert!(updated.resolved_at.is_some());
    }

    #[test]
    fn it_sets_updated_at() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let original_updated = task.updated_at;
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: Some("Changed".to_string()),
        description: None,
        phase: None,
        priority: None,
        status: None,
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert!(updated.updated_at >= original_updated);
    }

    #[test]
    fn it_unresolves_task_on_in_progress_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();
      store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: Some("in-progress".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        !dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.status, Status::InProgress);
      assert!(updated.resolved_at.is_none());
    }

    #[test]
    fn it_unresolves_task_on_non_terminal_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();
      store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: Some("open".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      assert!(dir.path().join("tasks/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml").exists());
      assert!(
        !dir
          .path()
          .join("tasks/resolved/zyxwvutsrqponmlkzyxwvutsrqponmlk.toml")
          .exists()
      );
      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.status, Status::Open);
      assert!(updated.resolved_at.is_none());
    }

    #[test]
    fn it_updates_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: None,
        description: None,
        phase: None,
        priority: None,
        status: Some("done".to_string()),
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.status, Status::Done);
    }

    #[test]
    fn it_updates_title_only() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        assigned_to: None,
        title: Some("New Title".to_string()),
        description: None,
        phase: None,
        priority: None,
        status: None,
        tags: None,
        metadata: vec![],
      };

      cmd.call(&config, &Theme::default()).unwrap();

      let updated = store::read_task(dir.path(), &task.id).unwrap();
      assert_eq!(updated.title, "New Title");
      assert_eq!(updated.description, "Original description");
      assert_eq!(updated.status, Status::Open);
      assert_eq!(updated.tags, vec!["original"]);
      assert_eq!(updated.links.len(), 1);
      assert_eq!(updated.metadata.get("priority").unwrap().as_str().unwrap(), "low");
    }
  }
}
