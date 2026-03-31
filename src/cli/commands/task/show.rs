use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::views::task::TaskDetailView,
};

/// Display a task's full details, description, and links.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Output task details as JSON.
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  /// Resolve the task by ID prefix and render its detail view.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(layout, &self.id, true)?;
    let task = store::read_task(layout, &id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&task)?;
      println!("{json}");
      return Ok(());
    }

    let id_str = task.id.to_string();
    let status_str = task.status.as_str();

    let link_strings: Vec<(String, String)> = task
      .links
      .iter()
      .map(|l| {
        let rel = l.rel.to_string();
        let full = l.ref_.rsplit('/').next().unwrap_or(&l.ref_);
        let target = if full.len() > 8 {
          full[..8].to_string()
        } else {
          full.to_string()
        };
        (rel, target)
      })
      .collect();

    let links: Vec<(&str, &str)> = link_strings.iter().map(|(r, t)| (r.as_str(), t.as_str())).collect();

    let body = if task.description.is_empty() {
      None
    } else {
      Some(task.description.as_str())
    };

    let view = TaskDetailView {
      id: &id_str,
      title: &task.title,
      status: status_str,
      priority: task.priority,
      phase: task.phase.map(|p| (p as u32, None)),
      assigned: task.assigned_to.as_deref(),
      tags: &task.tags,
      links,
      body,
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
    use super::*;

    #[test]
    fn it_resolves_resolved_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.description = "A test description".to_string();
      task.tags = vec!["rust".to_string()];
      task.links = vec![Link {
        ref_: "https://example.com".to_string(),
        rel: RelationshipType::RelatesTo,
      }];
      task.title = "Test Task".to_string();
      store::write_task(&ctx.layout, &task).unwrap();
      store::resolve_task(&ctx.layout, &task.id).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_task_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: true,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_task_detail() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
