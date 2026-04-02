use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Remove tags from any entity (task, artifact, or iteration) by ID prefix.
#[derive(Debug, Args)]
pub struct Command {
  /// Entity ID or unique prefix.
  pub id: String,
  /// Tags to remove (space or comma-separated).
  #[arg(value_delimiter = ',')]
  pub tags: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let resolved = store::resolve_any_id(&ctx.settings, &self.id)?;
    let params = store::TagParams {
      entity_type: resolved.entity_type,
      id_prefix: &self.id,
      tags: &self.tags,
    };
    let result = store::untag_entity(&ctx.settings, &params)?;
    let noun = resolved.entity_type;
    let msg = format!("Untagged {noun} {} from {}", result.id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, &ctx.theme));
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
    fn it_removes_tags_from_a_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      task.tags = vec!["rust".to_string(), "cli".to_string(), "keep".to_string()];
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.settings, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
