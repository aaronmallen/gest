use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{NewIteration, iteration::Status},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Create a new iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration title.
  pub title: String,
  /// Description text.
  #[arg(short, long)]
  pub description: Option<String>,
  /// Output the created iteration as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Key=value metadata pair (repeatable, e.g. `-m key=value`).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Print only the iteration ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
  /// Initial status: active, completed, or failed (default: active).
  #[arg(short, long)]
  pub status: Option<String>,
  /// Tag (repeatable, or comma-separated).
  // TODO: deprecate --tags in favor of --tag
  #[arg(long = "tag", value_delimiter = ',', alias = "tags")]
  pub tag: Vec<String>,
}

impl Command {
  /// Build a `NewIteration` from CLI args, persist it, and print confirmation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let status = match &self.status {
      Some(s) => s.parse::<Status>().map_err(cli::Error::InvalidInput)?,
      None => Status::Active,
    };

    let metadata = crate::cli::helpers::build_toml_metadata(&self.metadata)?;

    let tags = self.tag.clone();

    let new = NewIteration {
      description: self.description.clone().unwrap_or_default(),
      links: vec![],
      metadata,
      status,
      tags,
      tasks: vec![],
      title: self.title.clone(),
    };

    let iteration = store::create_iteration(config, new)?;

    if self.json {
      let json = serde_json::to_string_pretty(&iteration)?;
      println!("{json}");
      return Ok(());
    }

    if self.quiet {
      println!("{}", iteration.id);
      return Ok(());
    }

    let msg = format!("Created iteration {}", iteration.id);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{model::IterationFilter, test_helpers::make_test_context};

    #[test]
    fn it_creates_an_iteration_with_all_flags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Full Iteration".to_string(),
        description: Some("A description".to_string()),
        json: false,
        metadata: vec!["team=backend".to_string()],
        quiet: false,
        status: Some("active".to_string()),
        tag: vec!["sprint".to_string(), "q1".to_string()],
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.settings, &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Full Iteration");
      assert_eq!(iterations[0].description, "A description");
      assert_eq!(iterations[0].tags, vec!["sprint", "q1"]);
      assert_eq!(iterations[0].metadata.get("team").unwrap().as_str().unwrap(), "backend");
    }

    #[test]
    fn it_creates_an_iteration_with_defaults() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Sprint 1".to_string(),
        description: None,
        json: false,
        metadata: vec![],
        quiet: false,
        status: None,
        tag: vec![],
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.settings, &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].title, "Sprint 1");
      assert_eq!(iterations[0].status, Status::Active);
    }

    #[test]
    fn it_resolves_iteration_created_with_completed_status() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        title: "Done Iteration".to_string(),
        description: None,
        json: false,
        metadata: vec![],
        quiet: false,
        status: Some("completed".to_string()),
        tag: vec![],
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.settings, &filter).unwrap();
      assert_eq!(iterations.len(), 0);

      let filter = IterationFilter {
        all: true,
        ..Default::default()
      };
      let iterations = store::list_iterations(&ctx.settings, &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].status, Status::Completed);
      assert!(iterations[0].completed_at.is_some());
    }
  }
}
