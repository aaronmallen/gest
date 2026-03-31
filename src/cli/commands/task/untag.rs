use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Remove tags from a task.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Tags to remove (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  /// Remove the specified tags from the task and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(layout, &self.id, false)?;
    let mut task = store::read_task(layout, &id)?;

    super::super::tags::remove_tags(&mut task.tags, &self.tags);

    task.updated_at = Utc::now();
    store::write_task(layout, &task)?;

    let msg = format!("Untagged task {} from {}", id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_context, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_handles_nonexistent_tags_gracefully() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string()];
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.layout, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string()]);
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.layout, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
