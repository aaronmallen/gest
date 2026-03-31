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
  /// Key=value metadata pair (repeatable, e.g. `-m key=value`).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Initial status: active, completed, or failed (default: active).
  #[arg(short, long)]
  pub status: Option<String>,
  /// Comma-separated list of tags.
  #[arg(long)]
  pub tags: Option<String>,
}

impl Command {
  /// Build a `NewIteration` from CLI args, persist it, and print confirmation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let status = match &self.status {
      Some(s) => s.parse::<Status>().map_err(cli::Error::generic)?,
      None => Status::Active,
    };

    let metadata = {
      let pairs = crate::cli::helpers::split_key_value_pairs(&self.metadata)?;
      let mut table = toml::Table::new();
      for (key, value) in pairs {
        table.insert(key, toml::Value::String(value));
      }
      table
    };

    let tags = self
      .tags
      .as_deref()
      .map(crate::cli::helpers::parse_tags)
      .unwrap_or_default();

    let new = NewIteration {
      description: self.description.clone().unwrap_or_default(),
      links: vec![],
      metadata,
      status,
      tags,
      tasks: vec![],
      title: self.title.clone(),
    };

    let iteration = store::create_iteration(layout, new)?;

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
        metadata: vec!["team=backend".to_string()],
        status: Some("active".to_string()),
        tags: Some("sprint,q1".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.layout, &filter).unwrap();
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
        metadata: vec![],
        status: None,
        tags: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.layout, &filter).unwrap();
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
        metadata: vec![],
        status: Some("completed".to_string()),
        tags: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = IterationFilter::default();
      let iterations = store::list_iterations(&ctx.layout, &filter).unwrap();
      assert_eq!(iterations.len(), 0);

      let filter = IterationFilter {
        all: true,
        ..Default::default()
      };
      let iterations = store::list_iterations(&ctx.layout, &filter).unwrap();
      assert_eq!(iterations.len(), 1);
      assert_eq!(iterations[0].status, Status::Completed);
      assert!(iterations[0].completed_at.is_some());
    }
  }
}
