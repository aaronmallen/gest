use clap::Args;

use crate::{
  cli::{self, AppContext},
  store,
  ui::{composites::artifact_detail::ArtifactDetail, views::artifact::ArtifactDetailView},
};

/// Display an artifact's full details and rendered body.
#[derive(Debug, Args)]
pub struct Command {
  /// Artifact ID or unique prefix.
  pub id: String,
  /// Output as JSON instead of formatted detail.
  #[arg(short, long)]
  pub json: bool,
}

impl Command {
  /// Resolve the artifact and print its detail view or JSON representation.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let layout = &ctx.layout;
    let theme = &ctx.theme;
    let id = store::resolve_artifact_id(layout, &self.id, true)?;
    let artifact = store::read_artifact(layout, &id)?;

    if self.json {
      let json = serde_json::to_string_pretty(&artifact)?;
      println!("{json}");
      return Ok(());
    }

    let id_str = artifact.id.to_string();
    let created = artifact.created_at.format("%Y-%m-%d").to_string();
    let updated = artifact.updated_at.format("%Y-%m-%d").to_string();

    let body = if artifact.body.is_empty() {
      None
    } else {
      Some(artifact.body.as_str())
    };

    let detail = ArtifactDetail::new(&id_str, &artifact.title, &artifact.tags, &created, &updated, theme).body(body);
    let view = ArtifactDetailView::new(detail);
    println!("{view}");

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
    use super::*;

    #[test]
    fn it_shows_archived_artifact() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.layout, &artifact).unwrap();
      store::archive_artifact(&ctx.layout, &artifact.id).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_artifact_as_json() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: true,
      };

      cmd.call(&ctx).unwrap();
    }

    #[test]
    fn it_shows_artifact_detail() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());
      let mut artifact = make_test_artifact("zyxwvutsrqponmlkzyxwvutsrqponmlk");
      artifact.title = "Test Artifact".to_string();
      artifact.body = "# Hello\n\nSome content.".to_string();
      artifact.tags = vec!["spec".to_string()];
      store::write_artifact(&ctx.layout, &artifact).unwrap();

      let cmd = Command {
        id: "zyxw".to_string(),
        json: false,
      };

      cmd.call(&ctx).unwrap();
    }
  }
}
