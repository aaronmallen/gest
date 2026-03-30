use std::{collections::HashSet, path::Path};

use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{IterationFilter, iteration::Status},
  store,
  ui::{
    composites::empty_list::EmptyList,
    views::iteration::{IterationListData, IterationListView},
  },
};

/// List iterations, optionally filtered by status or tag.
#[derive(Debug, Args)]
pub struct Command {
  /// Output iteration list as JSON.
  #[arg(short, long)]
  pub json: bool,
  /// Include resolved (completed/failed) iterations.
  #[arg(short = 'a', long = "all")]
  pub show_all: bool,
  /// Filter by status: active, completed, or failed.
  #[arg(short, long)]
  pub status: Option<String>,
  /// Filter by tag.
  #[arg(long)]
  pub tag: Option<String>,
}

impl Command {
  /// Query iterations from the store and render as a table or JSON.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let data_dir = &ctx.data_dir;
    let theme = &ctx.theme;
    let status = match &self.status {
      Some(s) => Some(s.parse::<Status>().map_err(cli::Error::generic)?),
      None => None,
    };

    let filter = IterationFilter {
      all: self.show_all,
      status,
      tag: self.tag.clone(),
    };

    let iterations = store::list_iterations(data_dir, &filter)?;

    if self.json {
      let json = serde_json::to_string_pretty(&iterations)?;
      println!("{json}");
      return Ok(());
    }

    if iterations.is_empty() {
      println!("{}", EmptyList::new("iterations", theme));
      return Ok(());
    }

    let id_strings: Vec<String> = iterations.iter().map(|i| i.id.to_string()).collect();

    let view_data: Vec<IterationListData> = iterations
      .iter()
      .enumerate()
      .map(|(idx, i)| {
        let phase_count = compute_phase_count(data_dir, &i.tasks);
        IterationListData {
          id: &id_strings[idx],
          title: &i.title,
          phase_count,
          task_count: i.tasks.len(),
        }
      })
      .collect();

    println!("{}", IterationListView::new(view_data, theme));

    Ok(())
  }
}

/// Compute how many distinct phases the iteration's tasks span.
fn compute_phase_count(data_dir: &Path, tasks: &[String]) -> usize {
  let mut phases = HashSet::new();
  for task_ref in tasks {
    let task_id_str = task_ref.strip_prefix("tasks/").unwrap_or(task_ref);
    if let Ok(id) = task_id_str.parse()
      && let Ok(task) = store::read_task(data_dir, &id)
    {
      phases.insert(task.phase.unwrap_or(0));
    }
  }
  phases.len()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_context, make_test_iteration};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_filters_by_status() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      let mut i2 = make_test_iteration("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
      i2.title = "Failed one".to_string();
      i2.status = crate::model::iteration::Status::Failed;
      store::write_iteration(&ctx.data_dir, &i1).unwrap();
      store::write_iteration(&ctx.data_dir, &i2).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: Some("failed".to_string()),
        tag: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter {
        status: Some(Status::Failed),
        ..Default::default()
      };
      let iterations = store::list_iterations(&ctx.data_dir, &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Failed one");
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
    fn it_lists_iterations() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.data_dir, &i1).unwrap();

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
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.data_dir, &iteration).unwrap();

      let cmd = Command {
        show_all: false,
        json: true,
        status: None,
        tag: None,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
