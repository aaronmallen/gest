use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Add tags to a task, deduplicating with any existing tags.
#[derive(Debug, Args)]
pub struct Command {
  /// Task ID or unique prefix.
  pub id: String,
  /// Tags to add (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  /// Merge the given tags into the task and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_task_id(layout, &self.id, false)?;
    let mut task = store::read_task(layout, &id)?;

    super::super::tags::apply_tags(&mut task.tags, &self.tags);

    task.updated_at = Utc::now();
    store::write_task(layout, &task)?;

    let msg = format!("Tagged task {} with {}", id, self.tags.join(", "));
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
    fn it_adds_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.layout, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_deduplicates_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string()];
      store::write_task(&ctx.layout, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.layout, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }
  }
}
