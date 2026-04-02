use clap::Args;

use crate::{
  action,
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
  /// Replace all tags (repeatable, or comma-separated).
  // TODO: deprecate --tags in favor of --tag
  #[arg(long = "tag", value_delimiter = ',', alias = "tags")]
  pub tag: Vec<String>,
  /// New title.
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  /// Apply the provided patch fields to the iteration and persist the result.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(config, &self.id, true)?;

    let status = crate::cli::helpers::parse_optional_status::<Status>(self.status.as_deref())?;

    let metadata = {
      let existing = store::read_iteration(config, &id)?.metadata;
      crate::cli::helpers::merge_toml_metadata(&self.metadata, existing)?
    };

    let tags = if self.tag.is_empty() {
      None
    } else {
      Some(self.tag.clone())
    };

    let patch = IterationPatch {
      description: self.description.clone(),
      metadata,
      status,
      tags,
      title: self.title.clone(),
    };

    let author = action::resolve_author(false)?;
    let iteration = store::update_iteration(config, &id, patch, Some(&author))?;

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
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec!["team=backend".to_string()],
        status: None,
        tag: vec![],
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
      assert_eq!(updated.metadata.get("team").unwrap().as_str().unwrap(), "backend");
    }

    #[test]
    fn it_updates_status_to_completed() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec![],
        status: Some("completed".to_string()),
        tag: vec![],
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
      assert_eq!(updated.status, Status::Completed);
      assert!(updated.completed_at.is_some());
    }

    #[test]
    fn it_updates_title() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        description: None,
        metadata: vec![],
        status: None,
        tag: vec![],
        title: Some("New Title".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
      assert_eq!(updated.title, "New Title");
    }
  }
}
