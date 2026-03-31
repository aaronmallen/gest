use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Remove tags from an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Tags to remove (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  /// Remove the specified tags from the iteration's tag set.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_iteration_id(layout, &self.id, false)?;
    let mut iteration = store::read_iteration(layout, &id)?;

    super::super::tags::remove_tags(&mut iteration.tags, &self.tags);

    iteration.updated_at = Utc::now();
    store::write_iteration(layout, &iteration)?;

    let msg = format!("Untagged iteration {} from {}", id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, theme));
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
    fn it_handles_nonexistent_tags_gracefully() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tags = vec!["sprint".to_string()];
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(loaded.tags, vec!["sprint".to_string()]);
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tags = vec!["sprint".to_string(), "q1".to_string(), "keep".to_string()];
      store::write_iteration(&ctx.layout, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["sprint".to_string(), "q1".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.layout, &iteration.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
