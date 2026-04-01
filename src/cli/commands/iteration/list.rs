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
  /// Only show iterations that have at least one claimable task.
  #[arg(long)]
  pub has_available: bool,
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
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let status = crate::cli::helpers::parse_optional_status::<Status>(self.status.as_deref())?;

    let filter = IterationFilter {
      all: self.show_all,
      status,
      tag: self.tag.clone(),
    };

    let iterations = store::list_iterations(config, &filter)?;

    let iterations = if self.has_available {
      iterations
        .into_iter()
        .filter(|i| store::next_available_task(config, &i.id).ok().flatten().is_some())
        .collect()
    } else {
      iterations
    };

    if self.json {
      let json = serde_json::to_string_pretty(&iterations)?;
      println!("{json}");
      return Ok(());
    }

    if iterations.is_empty() {
      println!("{}", EmptyList::new("iterations", theme));
      return Ok(());
    }

    let view_data: Vec<IterationListData> = iterations
      .into_iter()
      .map(|i| {
        let phase_count = i.phase_count.unwrap_or(0);
        let task_count = i.tasks.len();
        IterationListData {
          id: i.id.to_string(),
          title: i.title,
          phase_count,
          task_count,
        }
      })
      .collect();

    println!("{}", IterationListView::new(view_data, theme));

    Ok(())
  }
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
      i2.status = Status::Failed;
      store::write_iteration(&ctx.settings, &i1).unwrap();
      store::write_iteration(&ctx.settings, &i2).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: Some("failed".to_string()),
        tag: None,
        has_available: false,
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter {
        status: Some(Status::Failed),
        ..Default::default()
      };
      let iterations = store::list_iterations(&ctx.settings, &filter).unwrap();
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
        has_available: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_lists_iterations() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let i1 = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &i1).unwrap();

      let cmd = Command {
        show_all: false,
        json: false,
        status: None,
        tag: None,
        has_available: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_outputs_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        show_all: false,
        json: true,
        status: None,
        tag: None,
        has_available: false,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
