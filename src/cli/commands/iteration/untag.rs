use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::EntityType,
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Remove tags from an iteration.
#[derive(Debug, Args)]
pub struct Command {
  /// Iteration ID or unique prefix.
  pub id: String,
  /// Tags to remove (space or comma-separated).
  #[arg(value_delimiter = ',')]
  pub tags: Vec<String>,
}

impl Command {
  /// Remove the specified tags from the iteration's tag set.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let params = store::TagParams {
      entity_type: EntityType::Iteration,
      id_prefix: &self.id,
      tags: &self.tags,
    };
    let result = store::untag_entity(&ctx.settings, &params)?;
    let msg = format!("Untagged iteration {} from {}", result.id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, &ctx.theme));
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
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
      assert_eq!(loaded.tags, vec!["sprint".to_string()]);
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut iteration = make_test_iteration("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      iteration.tags = vec!["sprint".to_string(), "q1".to_string(), "keep".to_string()];
      store::write_iteration(&ctx.settings, &iteration).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["sprint".to_string(), "q1".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_iteration(&ctx.settings, &iteration.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
