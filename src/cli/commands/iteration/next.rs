use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::Task,
  store,
  ui::views::task::TaskDetailView,
};

/// Find (or claim) the next available task in an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Claim the task (set to in-progress).
  #[arg(long)]
  pub claim: bool,
  /// Agent name for assignment (required with --claim).
  #[arg(long)]
  pub agent: Option<String>,
  /// Output as JSON.
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    if self.claim && self.agent.is_none() {
      return Err(cli::Error::generic("--agent is required when --claim is used"));
    }

    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, true)?;

    let task = match store::next_available_task(config, &id)? {
      Some(t) => t,
      None => return Err(cli::Error::no_result("no available tasks")),
    };

    let task = if self.claim {
      let agent = self.agent.as_deref().unwrap();
      store::claim_task(config, &task.id, agent)?
    } else {
      task
    };

    if self.json {
      let json = serde_json::to_string_pretty(&task)?;
      println!("{json}");
      return Ok(());
    }

    print_task_detail(&task, theme);
    Ok(())
  }
}

fn print_task_detail(task: &Task, theme: &crate::ui::theme::Theme) {
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    model::task::Status as TaskStatus,
    store,
    test_helpers::{make_test_context, make_test_iteration, make_test_task},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_no_result_when_no_tasks() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        claim: false,
        agent: None,
        json: false,
      };

      let err = cmd.call(&ctx).unwrap_err();
      assert_eq!(err.exit_code(), 2);
      assert_eq!(err.to_string(), "no available tasks");
    }

    #[test]
    fn it_peeks_at_next_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.phase = Some(1);
      task.priority = Some(1);
      store::write_task(&ctx.settings, &task).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks.push(task.id.to_string());
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        claim: false,
        agent: None,
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_claims_next_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.phase = Some(1);
      task.priority = Some(1);
      store::write_task(&ctx.settings, &task).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks.push(task.id.to_string());
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        claim: true,
        agent: Some("test-agent".to_string()),
        json: false,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_task(&ctx.settings, &task.id).unwrap();
      assert_eq!(updated.status, TaskStatus::InProgress);
      assert_eq!(updated.assigned_to, Some("test-agent".to_string()));
    }

    #[test]
    fn it_errors_when_claim_without_agent() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        claim: true,
        agent: None,
        json: false,
      };

      let err = cmd.call(&ctx).unwrap_err();
      assert_eq!(err.exit_code(), 1);
      assert!(err.to_string().contains("--agent is required"));
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let mut task = make_test_task("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      task.phase = Some(1);
      task.priority = Some(1);
      store::write_task(&ctx.settings, &task).unwrap();

      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tasks.push(task.id.to_string());
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        claim: false,
        agent: None,
        json: true,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
