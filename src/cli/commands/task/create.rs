use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{NewTask, task::Status},
  store,
  ui::views::task::TaskCreateView,
};

/// Create a new task with optional metadata, tags, and status.
#[derive(Debug, Args)]
pub struct Command {
  /// Task title.
  pub title: String,
  /// Actor assigned to this task.
  #[arg(long)]
  pub assigned_to: Option<String>,
  /// Description text (opens `$EDITOR` if omitted and stdin is a terminal).
  #[arg(short, long)]
  pub description: Option<String>,
  /// Key=value metadata pair (repeatable, e.g. `-m key=value`).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Execution phase for parallel grouping.
  #[arg(long)]
  pub phase: Option<u16>,
  /// Priority level (0-4, where 0 is highest).
  #[arg(short, long)]
  pub priority: Option<u8>,
  /// Initial status: open, in-progress, done, or cancelled (default: open).
  #[arg(short, long)]
  pub status: Option<String>,
  /// Comma-separated list of tags.
  #[arg(long)]
  pub tags: Option<String>,
}

impl Command {
  /// Persist a new task and print a confirmation view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let status = match &self.status {
      Some(s) => s.parse::<Status>().map_err(cli::Error::generic)?,
      None => Status::Open,
    };

    let metadata = crate::cli::helpers::build_toml_metadata(&self.metadata)?;

    let tags = self
      .tags
      .as_deref()
      .map(crate::cli::helpers::parse_tags)
      .unwrap_or_default();

    let description =
      crate::cli::helpers::read_from_editor(self.description.as_deref(), ".md", "Aborting: empty description")?;

    let new = NewTask {
      assigned_to: self.assigned_to.clone(),
      description,
      links: vec![],
      metadata,
      phase: self.phase,
      priority: self.priority,
      status,
      tags,
      title: self.title.clone(),
    };

    let task = store::create_task(config, new)?;

    let status_str = task.status.as_str();
    let fields = vec![("title", task.title.clone())];

    let view = TaskCreateView {
      id: &task.id.to_string(),
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

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::make_test_context;

    #[test]
    fn it_creates_a_task_with_all_flags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Full Task".to_string(),
        assigned_to: Some("agent-1".to_string()),
        description: Some("A description".to_string()),
        metadata: vec!["custom=high".to_string()],
        phase: Some(1),
        priority: Some(2),
        status: Some("in-progress".to_string()),
        tags: Some("rust,cli".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::TaskFilter::default();
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();

      assert_eq!(tasks.len(), 1);

      let task = &tasks[0];
      assert_eq!(task.title, "Full Task");
      assert_eq!(task.description, "A description");
      assert_eq!(task.status, Status::InProgress);
      assert_eq!(task.tags, vec!["rust", "cli"]);
      assert_eq!(task.assigned_to.as_deref(), Some("agent-1"));
      assert_eq!(task.phase, Some(1));
      assert_eq!(task.priority, Some(2));
      assert_eq!(task.links.len(), 0);
      assert_eq!(task.metadata.get("custom").unwrap().as_str().unwrap(), "high");
    }

    #[test]
    fn it_creates_a_task_with_defaults() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "My Task".to_string(),
        assigned_to: None,
        description: None,
        metadata: vec![],
        phase: None,
        priority: None,
        status: None,
        tags: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::TaskFilter::default();
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();

      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "My Task");
      assert_eq!(tasks[0].status, Status::Open);
      assert!(tasks[0].description.is_empty());
    }

    #[test]
    fn it_resolves_task_created_with_cancelled_status() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Cancelled Task".to_string(),
        assigned_to: None,
        description: Some("Cancelled".to_string()),
        metadata: vec![],
        phase: None,
        priority: None,
        status: Some("cancelled".to_string()),
        tags: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::TaskFilter::default();
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();
      assert_eq!(tasks.len(), 0);

      let filter = crate::model::TaskFilter {
        all: true,
        ..Default::default()
      };
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].status, Status::Cancelled);
      assert!(tasks[0].resolved_at.is_some());
    }

    #[test]
    fn it_resolves_task_created_with_done_status() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Done Task".to_string(),
        assigned_to: None,
        description: Some("Already done".to_string()),
        metadata: vec![],
        phase: None,
        priority: None,
        status: Some("done".to_string()),
        tags: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::TaskFilter::default();
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();
      assert_eq!(tasks.len(), 0);

      let filter = crate::model::TaskFilter {
        all: true,
        ..Default::default()
      };
      let tasks = store::list_tasks(&ctx.settings, &filter).unwrap();
      assert_eq!(tasks.len(), 1);
      assert_eq!(tasks[0].title, "Done Task");
      assert_eq!(tasks[0].status, Status::Done);
      assert!(tasks[0].resolved_at.is_some());
    }
  }
}
