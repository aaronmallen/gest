use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Remove tags from an artifact.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Tags to remove (space-separated).
  pub tags: Vec<String>,
}

impl Command {
  /// Remove the given tags from the artifact's tag list and persist.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_artifact_id(layout, &self.id, false)?;
    let mut artifact = store::read_artifact(layout, &id)?;

    super::super::tags::remove_tags(&mut artifact.tags, &self.tags);

    artifact.updated_at = Utc::now();
    store::write_artifact(layout, &artifact)?;

    let msg = format!("Untagged artifact {} from {}", id, self.tags.join(", "));
    println!("{}", SuccessMessage::new(&msg, theme));
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
    fn it_handles_nonexistent_tags_gracefully() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["spec".to_string()];
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["nonexistent".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["spec".to_string()]);
    }

    #[test]
    fn it_removes_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["spec".to_string(), "backend".to_string(), "keep".to_string()];
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        tags: vec!["spec".to_string(), "backend".to_string()],
      };
      cmd.call(&ctx).unwrap();

      let loaded = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
      assert_eq!(loaded.tags, vec!["keep".to_string()]);
    }
  }
}
