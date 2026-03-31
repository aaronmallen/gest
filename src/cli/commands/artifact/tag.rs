use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
};

/// Add tags to an artifact.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Tags to add (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  /// Merge the given tags into the artifact's tag list, deduplicate, and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    super::super::tags::tag_entity(
      ctx,
      &self.id,
      &self.tags,
      "artifact",
      store::resolve_artifact_id,
      store::read_artifact,
      |a| &mut a.tags,
      |a, t| a.updated_at = t,
      store::write_artifact,
    )
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
