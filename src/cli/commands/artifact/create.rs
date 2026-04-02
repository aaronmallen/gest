use clap::Args;

use crate::{
  cli::{self, AppContext},
  model::NewArtifact,
  store,
  ui::views::artifact::ArtifactCreateView,
};

/// Create a new artifact from inline text, a source file, an editor, or stdin.
#[derive(Debug, Args)]
pub struct Command {
  /// Body content as an inline string (skips editor and stdin).
  #[arg(short, long)]
  pub body: Option<String>,
  /// Artifact type (e.g. spec, adr, rfc, note).
  #[arg(short = 'k', long = "type")]
  pub kind: Option<String>,
  /// Key=value metadata pairs (repeatable).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Read body content from a file path.
  #[arg(short, long)]
  pub source: Option<String>,
  /// Tag (repeatable, or comma-separated).
  // TODO: deprecate --tags in favor of --tag
  #[arg(long = "tag", value_delimiter = ',', alias = "tags")]
  pub tag: Vec<String>,
  /// Artifact title (auto-extracted from first # heading if omitted).
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  /// Build a `NewArtifact`, persist it, and print the creation summary.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let theme = &ctx.theme;
    let metadata = crate::cli::helpers::build_yaml_metadata(&self.metadata)?;

    let tags = self.tag.clone();

    let body = if let Some(ref src) = self.source {
      std::fs::read_to_string(src).map_err(cli::Error::from)?
    } else {
      crate::cli::helpers::read_from_editor(self.body.as_deref(), ".md", "Aborting: empty body")?
    };

    let title = if let Some(ref t) = self.title {
      t.clone()
    } else {
      extract_title(&body)
        .ok_or_else(|| cli::Error::generic("No title found: body has no `# ` heading and no --title provided"))?
    };

    let new = NewArtifact {
      body,
      kind: self.kind.clone(),
      metadata,
      tags,
      title,
    };

    let artifact = store::create_artifact(config, new)?;

    let id_str = artifact.id.to_string();
    let mut view = ArtifactCreateView::new(&id_str, &artifact.title, theme);
    if let Some(ref src) = self.source {
      view = view.source(src);
    }
    println!("{view}");
    Ok(())
  }
}

fn extract_title(body: &str) -> Option<String> {
  for line in body.lines() {
    if let Some(rest) = line.strip_prefix("# ") {
      let title = rest.trim();
      if !title.is_empty() {
        return Some(title.to_string());
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::test_helpers::make_test_context;

    #[test]
    fn it_creates_an_artifact_from_source_file() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let source_path = dir.path().join("source.md");
      std::fs::write(&source_path, "# From File\n\nFile content.").unwrap();

      let cmd = Command {
        body: None,
        kind: None,
        metadata: vec![],
        source: Some(source_path.to_string_lossy().to_string()),
        tag: vec![],
        title: Some("Sourced Artifact".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::ArtifactFilter::default();
      let artifacts = store::list_artifacts(&ctx.settings, &filter).unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].body, "# From File\n\nFile content.");
    }

    #[test]
    fn it_creates_an_artifact_with_all_flags() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        body: Some("# Content\n\nSome body text.".to_string()),
        kind: Some("spec".to_string()),
        metadata: vec!["version=1".to_string()],
        source: None,
        tag: vec!["rust".to_string(), "cli".to_string()],
        title: Some("Full Artifact".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::ArtifactFilter::default();
      let artifacts = store::list_artifacts(&ctx.settings, &filter).unwrap();
      assert_eq!(artifacts.len(), 1);

      let artifact = &artifacts[0];
      assert_eq!(artifact.title, "Full Artifact");
      assert_eq!(artifact.body, "# Content\n\nSome body text.");
      assert_eq!(artifact.kind.as_deref(), Some("spec"));
      assert_eq!(artifact.tags, vec!["rust", "cli"]);
    }

    #[test]
    fn it_creates_an_artifact_with_defaults() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        body: None,
        kind: None,
        metadata: vec![],
        source: None,
        tag: vec![],
        title: Some("My Artifact".to_string()),
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::ArtifactFilter::default();
      let artifacts = store::list_artifacts(&ctx.settings, &filter).unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title, "My Artifact");
    }

    #[test]
    fn it_errors_when_no_title_and_no_heading() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        body: Some("No heading here".to_string()),
        kind: None,
        metadata: vec![],
        source: None,
        tag: vec![],
        title: None,
      };

      let result = cmd.call(&ctx);
      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("No title found"), "unexpected error: {err}");
    }

    #[test]
    fn it_extracts_title_from_body_when_title_omitted() {
      let dir = tempfile::tempdir().unwrap();
      let ctx = make_test_context(dir.path());

      let cmd = Command {
        body: Some("# Auto Title\n\nBody text.".to_string()),
        kind: None,
        metadata: vec![],
        source: None,
        tag: vec![],
        title: None,
      };

      cmd.call(&ctx).unwrap();

      let filter = crate::model::ArtifactFilter::default();
      let artifacts = store::list_artifacts(&ctx.settings, &filter).unwrap();
      assert_eq!(artifacts.len(), 1);
      assert_eq!(artifacts[0].title, "Auto Title");
    }
  }

  mod extract_title {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_extracts_first_h1_heading() {
      let body = "Some preamble\n# My Title\n\nBody text";
      assert_eq!(extract_title(body), Some("My Title".to_string()));
    }

    #[test]
    fn it_ignores_h2_headings() {
      let body = "## Not a title\n# Real Title";
      assert_eq!(extract_title(body), Some("Real Title".to_string()));
    }

    #[test]
    fn it_returns_none_when_no_heading() {
      let body = "No heading here\nJust text";
      assert_eq!(extract_title(body), None);
    }

    #[test]
    fn it_skips_empty_h1() {
      let body = "# \n# Actual Title";
      assert_eq!(extract_title(body), Some("Actual Title".to_string()));
    }

    #[test]
    fn it_trims_whitespace_from_title() {
      let body = "#   Spaced Title  \n";
      assert_eq!(extract_title(body), Some("Spaced Title".to_string()));
    }
  }
}
