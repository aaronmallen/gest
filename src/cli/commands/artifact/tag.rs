use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::EntityType,
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Add tags to an artifact.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Tags to add (space or comma-separated).
  #[arg(value_delimiter = ',')]
  pub tags: Vec<String>,
}

impl Command {
  /// Merge the given tags into the artifact's tag list, deduplicate, and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let params = store::TagParams {
      entity_type: EntityType::Artifact,
      id_prefix: &self.id,
      tags: &self.tags,
    };
    let result = store::tag_entity(&ctx.settings, &params)?;
    let msg = format!("Tagged artifact {} with {}", result.id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, &ctx.theme));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helpers::{make_test_artifact, make_test_context};

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_adds_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["spec".to_string(), "backend".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.settings, &artifact.id).unwrap();

      assert_eq!(loaded.tags, vec!["spec".to_string(), "backend".to_string()]);
    }

    #[test]
    fn it_deduplicates_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["spec".to_string()];
      store::write_artifact(&ctx.settings, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["spec".to_string(), "backend".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.settings, &artifact.id).unwrap();

      assert_eq!(loaded.tags, vec!["spec".to_string(), "backend".to_string()]);
    }
  }
}
