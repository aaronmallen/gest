use clap::Args;
use yansi::Paint;

use crate::{
  config,
  config::Config,
  model::{
    TaskFilter,
    link::RelationshipType,
    task::{Status, Task},
  },
  store,
  ui::{
    components::{EmptyList, Group, GroupedList},
    theme::Theme,
    utils::{format_id, shortest_unique_prefixes},
  },
};

/// Status group ordering for display.
const STATUS_ORDER: &[Status] = &[Status::Open, Status::InProgress, Status::Done, Status::Cancelled];

/// List tasks grouped by status, optionally filtered
#[derive(Debug, Args)]
pub struct Command {
  /// Output task list as JSON
  #[arg(short, long)]
  pub json: bool,
  /// Include resolved (done/cancelled) tasks
  #[arg(short = 'a', long = "all")]
  pub show_all: bool,
  /// Filter by status: open, in-progress, done, or cancelled
  #[arg(short, long)]
  pub status: Option<String>,
  /// Filter by tag
  #[arg(long)]
  pub tag: Option<String>,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    let status = match &self.status {
      Some(s) => Some(s.parse::<Status>().map_err(crate::Error::generic)?),
      None => None,
    };

    let filter = TaskFilter {
      all: self.show_all,
      status,
      tag: self.tag.clone(),
    };

    let data_dir = config::data_dir(config)?;
    let tasks = store::list_tasks(&data_dir, &filter)?;

    if self.json {
      let json = serde_json::to_string_pretty(&tasks)?;
      println!("{json}");
      return Ok(());
    }

    if tasks.is_empty() {
      EmptyList::new("tasks").write_to(&mut std::io::stdout())?;
      return Ok(());
    }

    // Compute shortest unique ID prefixes across all tasks
    let id_strings: Vec<String> = tasks.iter().map(|t| t.id.to_string()).collect();
    let prefix_lens = shortest_unique_prefixes(&id_strings);

    // Build a lookup from task id -> prefix_len
    let prefix_map: std::collections::HashMap<String, usize> = id_strings
      .iter()
      .zip(&prefix_lens)
      .map(|(id, &len)| (id.clone(), len))
      .collect();

    // Group tasks by status
    let groups: Vec<Group> = STATUS_ORDER
      .iter()
      .map(|status| {
        let mut matching: Vec<&Task> = tasks.iter().filter(|t| &t.status == status).collect();
        // Sort by creation date, oldest first
        matching.sort_by_key(|t| t.created_at);

        let rows: Vec<Vec<String>> = matching
          .iter()
          .map(|task| {
            let id_str = task.id.to_string();
            let prefix_len = prefix_map.get(&id_str).copied().unwrap_or(1);
            let id_cell = format_id(&task.id, prefix_len, Some(8), theme);

            // Title with optional status marker for resolved tasks
            let status_marker = if task.resolved_at.is_some() {
              match task.status {
                Status::Done => " (done)",
                Status::Cancelled => " (cancelled)",
                _ => "",
              }
            } else {
              ""
            };
            let title_cell = format!("{}{}", task.title, status_marker);

            // Tags as @tag_name styled with tag
            let tags_cell = task
              .tags
              .iter()
              .map(|tag| format!("@{}", tag).paint(theme.tag).to_string())
              .collect::<Vec<_>>()
              .join(" ");

            // Indicators: blocked (!!), blocking (⚠ N)
            let mut indicators = Vec::new();

            let has_blocked_by = task.links.iter().any(|l| l.rel == RelationshipType::BlockedBy);
            if has_blocked_by {
              indicators.push("!!".paint(theme.error).to_string());
            }

            let blocks_count = task.links.iter().filter(|l| l.rel == RelationshipType::Blocks).count();
            if blocks_count > 0 {
              indicators.push(format!("\u{26a0} {}", blocks_count).paint(theme.emphasis).to_string());
            }

            let indicators_cell = indicators.join(" ");

            vec![id_cell, title_cell, tags_cell, indicators_cell]
          })
          .collect();

        Group::new(status_heading(status), rows)
      })
      .collect();

    GroupedList::new(groups, theme).write_to(&mut std::io::stdout())?;

    Ok(())
  }
}

fn status_heading(status: &Status) -> &'static str {
  match status {
    Status::Open => "Open",
    Status::InProgress => "In Progress",
    Status::Done => "Done",
    Status::Cancelled => "Cancelled",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::link::Link,
    store,
    test_helpers::{make_test_config, make_test_task},
  };

  /// Create a test task with a specific title and status, using shared helpers.
  fn make_task_with(id: &str, title: &str, status: Status) -> Task {
    let mut task = make_test_task(id);
    task.title = title.to_string();
    task.status = status;
    task
  }

  mod call {
    use super::*;

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      store::write_task(
        dir.path(),
        &make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Open", Status::Open),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task_with("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "InProg", Status::InProgress),
      )
      .unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: Some("in-progress".to_string()),
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_handles_empty_list() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_includes_resolved_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved", Status::Open);
      store::write_task(dir.path(), &task).unwrap();
      store::resolve_task(dir.path(), &task.id).unwrap();

      let cmd = Command {
        show_all: true,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_lists_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task One", Status::Open);
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      let task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "JSON Task", Status::Open);
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: true,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_groups_tasks_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());

      store::write_task(
        dir.path(),
        &make_task_with("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkku", "Open task", Status::Open),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task_with(
          "lllllllllllllllllllllllllllllllu",
          "In progress task",
          Status::InProgress,
        ),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task_with("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Done task", Status::Done),
      )
      .unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_shows_blocked_indicator() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());

      let mut task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked task", Status::Open);
      task.links = vec![Link {
        ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        rel: RelationshipType::BlockedBy,
      }];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_shows_blocking_indicator() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());

      let mut task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocking task", Status::Open);
      task.links = vec![
        Link {
          ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkku".to_string(),
          rel: RelationshipType::Blocks,
        },
        Link {
          ref_: "tasks/lllllllllllllllllllllllllllllllu".to_string(),
          rel: RelationshipType::Blocks,
        },
      ];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }

    #[test]
    fn it_renders_tags_with_at_prefix() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());

      let mut task = make_task_with("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged task", Status::Open);
      task.tags = vec!["bug".to_string(), "urgent".to_string()];
      store::write_task(dir.path(), &task).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
      };

      cmd.call(&config, &Theme::default()).unwrap();
    }
  }
}
