use std::io::{BufRead, IsTerminal};

use clap::Args;
use serde::Deserialize;

use crate::{
  action,
  cli::{self, AppContext},
  model::{NewTask, Task, link::RelationshipType, task::Status},
  store,
  ui::views::task::TaskCreateView,
};

/// Create a new task with optional metadata, tags, and status.
#[derive(Debug, Args)]
pub struct Command {
  /// Task title.
  #[arg(required_unless_present = "batch")]
  pub title: Option<String>,
  /// Actor assigned to this task.
  #[arg(long)]
  pub assigned_to: Option<String>,
  /// Read NDJSON from stdin (one task per line).
  #[arg(long, conflicts_with_all = ["title", "assigned_to", "description", "iteration", "link", "metadata", "phase", "priority", "status", "tag"])]
  pub batch: bool,
  /// Description text (opens `$EDITOR` if omitted and stdin is a terminal).
  #[arg(short, long)]
  pub description: Option<String>,
  /// Add the task to an iteration (ID or prefix).
  #[arg(short, long)]
  pub iteration: Option<String>,
  /// Output the created task as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Create a link on the new task (repeatable, format: `<rel>:<target_id>`).
  #[arg(short, long)]
  pub link: Vec<String>,
  /// Key=value metadata pair (repeatable, e.g. `-m key=value`).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Execution phase for parallel grouping.
  #[arg(long)]
  pub phase: Option<u16>,
  /// Priority level (0-4, where 0 is highest).
  #[arg(short, long)]
  pub priority: Option<u8>,
  /// Print only the task ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
  /// Initial status: open, in-progress, done, or cancelled (default: open).
  #[arg(short, long)]
  pub status: Option<String>,
  /// Tag (repeatable, or comma-separated).
  // TODO: deprecate --tags in favor of --tag
  #[arg(long = "tag", value_delimiter = ',', alias = "tags")]
  pub tag: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BatchTaskInput {
  title: String,
  #[serde(default)]
  assigned_to: Option<String>,
  #[serde(default)]
  description: Option<String>,
  #[serde(default)]
  iteration: Option<String>,
  #[serde(default)]
  links: Vec<String>,
  #[serde(default)]
  metadata: std::collections::HashMap<String, serde_json::Value>,
  #[serde(default)]
  phase: Option<u16>,
  #[serde(default)]
  priority: Option<u8>,
  #[serde(default)]
  status: Option<String>,
  #[serde(default)]
  tags: Vec<String>,
}

impl Command {
  /// Persist a new task and print a confirmation view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    if self.batch {
      return self.batch_call(ctx);
    }

    let config = &ctx.settings;
    let theme = &ctx.theme;
    let title = self.title.clone().unwrap_or_default();
    let status = match &self.status {
      Some(s) => s.parse::<Status>().map_err(cli::Error::InvalidInput)?,
      None => Status::Open,
    };

    let metadata = crate::cli::helpers::build_toml_metadata(&self.metadata)?;

    let tags = self.tag.clone();

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
      title,
    };

    let task = store::create_task(config, new)?;
    let task_id_str = task.id.to_string();

    // Process --link flags
    for link_arg in &self.link {
      process_link(config, &task_id_str, link_arg)?;
    }

    // Process --iteration flag
    if let Some(ref iter_prefix) = self.iteration {
      let iter_id = store::resolve_iteration_id(config, iter_prefix, false)?;
      let task_ref = format!("tasks/{}", task.id);
      store::add_iteration_task(config, &iter_id, &task_ref)?;
    }

    // Re-read task if links were added (so JSON output includes them)
    let task = if !self.link.is_empty() {
      store::read_task(config, &task.id)?
    } else {
      task
    };

    if self.json {
      let json = serde_json::to_string_pretty(&task)?;
      println!("{json}");
      return Ok(());
    }

    if self.quiet {
      println!("{}", task.id);
      return Ok(());
    }

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

  fn batch_call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let stdin = std::io::stdin();

    if stdin.is_terminal() {
      return Err(cli::Error::InvalidInput("--batch requires piped stdin".into()));
    }

    for (line_num, line) in stdin.lock().lines().enumerate() {
      let line = line.map_err(|e| cli::Error::InvalidInput(format!("line {}: {e}", line_num + 1)))?;
      if line.trim().is_empty() {
        continue;
      }

      let input: BatchTaskInput =
        serde_json::from_str(&line).map_err(|e| cli::Error::InvalidInput(format!("line {}: {e}", line_num + 1)))?;

      let status = match &input.status {
        Some(s) => s.parse::<Status>().map_err(cli::Error::InvalidInput)?,
        None => Status::Open,
      };

      let mut metadata = toml::Table::new();
      for (k, v) in &input.metadata {
        metadata.insert(k.clone(), json_value_to_toml(v));
      }

      let new = NewTask {
        assigned_to: input.assigned_to,
        description: input.description.unwrap_or_default(),
        links: vec![],
        metadata,
        phase: input.phase,
        priority: input.priority,
        status,
        tags: input.tags,
        title: input.title,
      };

      let task = store::create_task(config, new)?;
      let task_id_str = task.id.to_string();

      for link_arg in &input.links {
        process_link(config, &task_id_str, link_arg)?;
      }

      if let Some(ref iter_prefix) = input.iteration {
        let iter_id = store::resolve_iteration_id(config, iter_prefix, false)?;
        let task_ref = format!("tasks/{}", task.id);
        store::add_iteration_task(config, &iter_id, &task_ref)?;
      }

      let task = if !input.links.is_empty() {
        store::read_task(config, &task.id)?
      } else {
        task
      };

      if self.quiet {
        println!("{}", task.id);
      } else {
        let json = serde_json::to_string(&task)?;
        println!("{json}");
      }
    }

    Ok(())
  }
}

fn json_value_to_toml(v: &serde_json::Value) -> toml::Value {
  match v {
    serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
    serde_json::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        toml::Value::Integer(i)
      } else if let Some(f) = n.as_f64() {
        toml::Value::Float(f)
      } else {
        toml::Value::String(n.to_string())
      }
    }
    serde_json::Value::String(s) => toml::Value::String(s.clone()),
    _ => toml::Value::String(v.to_string()),
  }
}

fn process_link(config: &crate::config::Settings, task_id_str: &str, link_arg: &str) -> cli::Result<()> {
  let (rel_str, target_id) = link_arg.split_once(':').ok_or_else(|| {
    cli::Error::InvalidInput(format!(
      "Invalid --link format '{link_arg}', expected <rel>:<target_id>"
    ))
  })?;
  let rel: RelationshipType = rel_str.parse().map_err(|e: String| cli::Error::InvalidInput(e))?;
  let is_artifact = store::resolve_artifact_id(config, target_id, true).is_ok()
    && store::resolve_task_id(config, target_id, true).is_err();
  action::link::link::<Task>(config, task_id_str, target_id, &rel, is_artifact)?;
  Ok(())
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
        title: Some("Full Task".to_string()),
        assigned_to: Some("agent-1".to_string()),
        batch: false,
        description: Some("A description".to_string()),
        iteration: None,
        json: false,
        link: vec![],
        metadata: vec!["custom=high".to_string()],
        phase: Some(1),
        priority: Some(2),
        quiet: false,
        status: Some("in-progress".to_string()),
        tag: vec!["rust".to_string(), "cli".to_string()],
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
        title: Some("My Task".to_string()),
        assigned_to: None,
        batch: false,
        description: None,
        iteration: None,
        json: false,
        link: vec![],
        metadata: vec![],
        phase: None,
        priority: None,
        quiet: false,
        status: None,
        tag: vec![],
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
        title: Some("Cancelled Task".to_string()),
        assigned_to: None,
        batch: false,
        description: Some("Cancelled".to_string()),
        iteration: None,
        json: false,
        link: vec![],
        metadata: vec![],
        phase: None,
        priority: None,
        quiet: false,
        status: Some("cancelled".to_string()),
        tag: vec![],
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
        title: Some("Done Task".to_string()),
        assigned_to: None,
        batch: false,
        description: Some("Already done".to_string()),
        iteration: None,
        json: false,
        link: vec![],
        metadata: vec![],
        phase: None,
        priority: None,
        quiet: false,
        status: Some("done".to_string()),
        tag: vec![],
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
