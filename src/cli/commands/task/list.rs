use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{TaskFilter, task::Status},
  store,
  store::ResolvedBlocking,
  ui::{
    composites::empty_list::EmptyList,
    views::task::{TaskListView, TaskViewData},
  },
};

/// List tasks, optionally filtered by status or tag.
#[derive(Debug, Args)]
pub struct Command {
  /// Output task list as JSON.
  #[arg(short, long)]
  pub json: bool,
  /// Include resolved (done/cancelled) tasks.
  #[arg(short = 'a', long = "all")]
  pub show_all: bool,
  /// Filter by status: open, in-progress, done, or cancelled.
  #[arg(short, long)]
  pub status: Option<String>,
  /// Filter by tag.
  #[arg(long)]
  pub tag: Option<String>,
}

impl Command {
  /// Fetch and display tasks, rendering as JSON or a themed list view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let status = match &self.status {
      Some(s) => Some(s.parse::<Status>().map_err(cli::Error::generic)?),
      None => None,
    };

    let filter = TaskFilter {
      all: self.show_all,
      status,
      tag: self.tag.clone(),
    };

    let tasks = store::list_tasks(layout, &filter)?;

    if self.json {
      let json = serde_json::to_string_pretty(&tasks)?;
      println!("{json}");
      return Ok(());
    }

    if tasks.is_empty() {
      println!("{}", EmptyList::new("tasks", theme));
      return Ok(());
    }

    let resolved: Vec<ResolvedBlocking> = store::resolve_blocking_batch(layout, &tasks);

    let view_data: Vec<TaskViewData> = tasks
      .into_iter()
      .enumerate()
      .map(|(i, t)| TaskViewData {
        status: t.status.as_str().into(),
        id: t.id.to_string(),
        title: t.title,
        priority: t.priority,
        tags: t.tags,
        is_blocking: resolved[i].is_blocking,
        blocked_by: resolved[i].blocked_by_ids.first().cloned(),
      })
      .collect();

    println!("{}", TaskListView::new(view_data, theme));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::{
      link::{Link, RelationshipType},
      task::Status,
    },
    store,
    test_helpers::{make_test_context, make_test_task},
  };

  mod call {
    use super::*;

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      store::write_task(
        &ctx.layout,
        &make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Open", Status::Open),
      )
      .unwrap();
      store::write_task(
        &ctx.layout,
        &make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "InProg", Status::InProgress),
      )
      .unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: Some("in-progress".to_string()),
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_handles_empty_list() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_lists_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task One", Status::Open);
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "JSON Task", Status::Open);
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: true,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_blocked_indicator() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked task", Status::Open);
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_blocking_indicator() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocking task", Status::Open);
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::Blocks,
      }];
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }
  }

  fn make_task(id: &str, title: &str, status: Status) -> crate::model::Task {
    crate::model::Task {
      title: title.to_string(),
      status,
      ..make_test_task(id)
    }
  }
}
