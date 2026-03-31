use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{TaskPatch, task::Status},
  store,
  ui::views::task::TaskUpdateView,
};

/// Update a task's title, description, status, tags, or metadata.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Actor assigned to this task.
  #[arg(long)]
  pub assigned_to: Option<String>,
  /// New description text.
  #[arg(short, long)]
  pub description: Option<String>,
  /// Key=value metadata pair, merged with existing (repeatable).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Execution phase for parallel grouping.
  #[arg(long)]
  pub phase: Option<u16>,
  /// Priority level (0-4, where 0 is highest).
  #[arg(short, long)]
  pub priority: Option<u8>,
  /// New status (done/cancelled auto-resolves; open/in-progress un-resolves).
  #[arg(short, long)]
  pub status: Option<String>,
  /// Replace all tags with this comma-separated list.
  #[arg(long)]
  pub tags: Option<String>,
  /// New title.
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  /// Apply the patch to an existing task and print the confirmation view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(config, &self.id, true)?;

    let description = self.description.clone();

    let status = crate::cli::helpers::parse_optional_status::<Status>(self.status.as_deref())?;

    let metadata = {
      let existing = store::read_task(config, &id)?.metadata;
      crate::cli::helpers::merge_toml_metadata(&self.metadata, existing)?
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

    let task = store::update_task(config, &id, patch)?;
    let id_str = task.id.to_string();

    let status_str = if self.status.is_some() {
      Some(task.status.as_str())
    } else {
      None
    };
    let mut fields = Vec::new();
    if self.assigned_to.is_some() {
      fields.push(("assigned", task.assigned_to.as_deref().unwrap_or("").to_string()));
    }

    let view = TaskUpdateView {
      id: &id_str,
      fields,
      status: status_str,
      theme,
    };
    println!("{view}");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::link::{Link, RelationshipType},
    store,
    test_helpers::{make_test_context, make_test_task},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_metadata_entries() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

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

      cmd.call(&ctx).unwrap();

      let updated = store::read_task(&ctx.settings, &task.id).unwrap();

      assert_eq!(updated.metadata.get("priority").unwrap().as_str().unwrap(), "low");
      assert_eq!(updated.metadata.get("team").unwrap().as_str().unwrap(), "backend");
    }

    #[test]
    fn it_preserves_links_and_metadata() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

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

      cmd.call(&ctx).unwrap();

      let updated = store::read_task(&ctx.settings, &task.id).unwrap();

      assert_eq!(updated.links.len(), 1);
      assert_eq!(updated.links[0].rel, RelationshipType::RelatesTo);
      assert_eq!(updated.metadata.get("priority").unwrap().as_str().unwrap(), "low");
    }

    #[test]
    fn it_updates_title_only() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_rich_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

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

      cmd.call(&ctx).unwrap();

      let updated = store::read_task(&ctx.settings, &task.id).unwrap();

      assert_eq!(updated.title, "New Title");
      assert_eq!(updated.description, "Original description");
      assert_eq!(updated.status, Status::Open);
      assert_eq!(updated.tags, vec!["original"]);
    }
  }

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
}
