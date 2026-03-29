use clap::Args;

use crate::{
  config,
  config::Config,
  model::{
    TaskFilter,
    task::{STATUS_ORDER, Status, Task},
  },
  store,
  ui::{
    components::{EmptyList, Group, GroupedList, Indicators},
    theme::Theme,
    utils::{format_id, format_tags, shortest_unique_prefixes},
  },
};

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
        matching.sort_by_key(|t| t.created_at);
        let rows: Vec<Vec<String>> = matching
          .iter()
          .map(|task| build_row(task, &prefix_map, theme))
          .collect();
        Group::new(status_heading(status), rows)
      })
      .collect();

    GroupedList::new(groups, theme).write_to(&mut std::io::stdout())?;

    Ok(())
  }
}

fn build_row(task: &Task, prefix_map: &std::collections::HashMap<String, usize>, theme: &Theme) -> Vec<String> {
  let id_str = task.id.to_string();
  let prefix_len = prefix_map.get(&id_str).copied().unwrap_or(1);
  let id_cell = format_id(&task.id, prefix_len, Some(8), theme);

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
  let tags_cell = format_tags(&task.tags, theme);

  let indicators_cell = Indicators::new(&task.links, theme).to_string();

  vec![id_cell, title_cell, tags_cell, indicators_cell]
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
    model::{
      Task,
      link::{Link, RelationshipType},
      task::Status,
    },
    store,
    test_helpers::{make_test_config, make_test_task},
  };

  mod call {
    use super::*;

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let config = make_test_config(dir.path());
      store::write_task(
        dir.path(),
        &make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Open", Status::Open),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "InProg", Status::InProgress),
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
      let task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Resolved", Status::Open);
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
      let task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Task One", Status::Open);
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
      let task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "JSON Task", Status::Open);
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
        &make_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkku", "Open task", Status::Open),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task(
          "lllllllllllllllllllllllllllllllu",
          "In progress task",
          Status::InProgress,
        ),
      )
      .unwrap();
      store::write_task(
        dir.path(),
        &make_task("mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm", "Done task", Status::Done),
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

      let mut task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocked task", Status::Open);
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

      let mut task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Blocking task", Status::Open);
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

      let mut task = make_task("zyxwvutsrqponmlkzyxwvutsrqponmlk", "Tagged task", Status::Open);
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

  fn make_task(id: &str, title: &str, status: Status) -> Task {
    Task {
      title: title.to_string(),
      status,
      ..make_test_task(id)
    }
  }
}
