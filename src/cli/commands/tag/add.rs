use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Add tags to any entity (task, artifact, or iteration) by ID prefix.
#[derive(Debug, Args)]
pub struct Command {
  /// Entity ID or unique prefix.
  pub id: String,
  /// Tags to add (space or comma-separated).
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
    let result = store::tag_entity(&ctx.settings, &params)?;
    let noun = resolved.entity_type;
    let msg = format!("Tagged {noun} {} with {}", result.id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, &ctx.theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_artifact, make_test_context, make_test_task};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_tags_a_task() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let task = make_test_task("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_task(&ctx.settings, &task).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["rust".to_string(), "cli".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_task(&ctx.settings, &task.id).unwrap();
      assert_eq!(loaded.tags, vec!["rust".to_string(), "cli".to_string()]);
    }

    #[test]
    fn it_tags_an_artifact() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["spec".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.settings, &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["spec".to_string()]);
    }
  }
}
