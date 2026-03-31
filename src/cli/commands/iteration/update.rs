use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::{IterationPatch, iteration::Status},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Update an iteration's title, description, status, tags, or metadata.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// New description.
  #[arg(short, long)]
  pub description: Option<String>,
  /// Key=value metadata pair, merged with existing (repeatable, e.g. `-m key=value`).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// New status: active, completed, or failed.
  #[arg(short, long)]
  pub status: Option<String>,
  /// Replace all tags with this comma-separated list.
  #[arg(long)]
  pub tags: Option<String>,
  /// New title.
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  /// Apply the provided patch fields to the iteration and persist the result.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(layout, &self.id, true)?;

    let status = self
      .status
      .as_deref()
      .map(|s| s.parse::<Status>().map_err(cli::Error::generic))
      .transpose()?;

    let metadata = if self.metadata.is_empty() {
      None
    } else {
      let pairs = crate::cli::helpers::split_key_value_pairs(&self.metadata)?;
      let mut table = store::read_iteration(layout, &id)?.metadata;
      for (key, value) in pairs {
        table.insert(key, toml::Value::String(value));
      }
      Some(table)
    };

    let tags = self.tags.as_deref().map(crate::cli::helpers::parse_tags);

    let patch = IterationPatch {
      description: self.description.clone(),
      metadata,
      status,
      tags,
      title: self.title.clone(),
    };

    let iteration = store::update_iteration(layout, &id, patch)?;

    let msg = format!("Updated iteration {}", iteration.id);
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_context, make_test_iteration},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_metadata() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec!["team=backend".to_string()],
        status: None,
        tags: None,
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(updated.metadata.get("team").unwrap().as_str().unwrap(), "backend");
    }

    #[test]
    fn it_updates_status_to_completed() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec![],
        status: Some("completed".to_string()),
        tags: None,
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(updated.status, Status::Completed);
      assert!(updated.completed_at.is_some());
    }

    #[test]
    fn it_updates_title() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec![],
        status: None,
        tags: None,
        title: Some("New Title".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(updated.title, "New Title");
    }
  }
}
