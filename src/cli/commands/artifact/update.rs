use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::ArtifactPatch,
  store,
  ui::composites::success_message::SuccessMessage,
};

/// Update an artifact's title, body, type, or tags.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Replace the body content.
  #[arg(short, long)]
  pub body: Option<String>,
  /// Artifact type (e.g. spec, adr, rfc, note).
  #[arg(short = 'k', long = "type")]
  pub kind: Option<String>,
  /// Replace all tags with this comma-separated list.
  #[arg(long)]
  pub tags: Option<String>,
  /// New title.
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  /// Apply the provided patch fields to the artifact and print a summary of changes.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_artifact_id(layout, &self.id, true)?;

    let tags = self.tags.as_deref().map(crate::cli::helpers::parse_tags);

    let patch = ArtifactPatch {
      body: self.body.clone(),
      kind: self.kind.clone(),
      metadata: None,
      tags,
      title: self.title.clone(),
    };

    let artifact = store::update_artifact(layout, &id, patch)?;
    let id_str = artifact.id.to_string();

    let mut msg = SuccessMessage::new("updated artifact", theme).id(&id_str);
    if self.title.is_some() {
      msg = msg.field("title", &artifact.title);
    }
    if self.kind.is_some() {
      msg = msg.field("type", artifact.kind.as_deref().unwrap_or(""));
    }
    println!("{msg}");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    store,
    test_helpers::{make_test_artifact, make_test_context},
  };

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_updates_body() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.body = "Old body".to_string();
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        body: Some("New body".to_string()),
        kind: None,
        tags: None,
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
      assert_eq!(updated.body, "New body");
    }

    #[test]
    fn it_updates_tags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.tags = vec!["old".to_string()];
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        body: None,
        kind: None,
        tags: Some("new,tags".to_string()),
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
      assert_eq!(updated.tags, vec!["new", "tags"]);
    }

    #[test]
    fn it_updates_title_only() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.title = "Original Title".to_string();
      artifact.body = "Original body".to_string();
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        body: None,
        kind: None,
        tags: None,
        title: Some("New Title".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let updated = store::read_artifact(&ctx.layout, &artifact.id).unwrap();
      assert_eq!(updated.title, "New Title");
      assert_eq!(updated.body, "Original body");
    }
  }
}
