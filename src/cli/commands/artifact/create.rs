use std::io::{BufRead, IsTerminal};

use clap::Args;
use serde::Deserialize;

use crate::{
  cli::{self, AppContext},
  model::NewArtifact,
  store,
  ui::views::artifact::ArtifactCreateView,
};

/// Create a new artifact from inline text, a source file, an editor, or stdin.
#[derive(Debug, Args)]
pub struct Command {
  /// Read NDJSON from stdin (one artifact per line).
  #[arg(long, conflicts_with_all = ["body", "iteration", "kind", "metadata", "source", "tag", "title"])]
  pub batch: bool,
  /// Body content as an inline string (skips editor and stdin).
  #[arg(short, long)]
  pub body: Option<String>,
  /// Add the artifact to an iteration (ID or prefix).
  #[arg(short, long)]
  pub iteration: Option<String>,
  /// Output the created artifact as JSON.
  #[arg(short, long, conflicts_with = "quiet")]
  pub json: bool,
  /// Artifact type (e.g. spec, adr, rfc, note).
  #[arg(short = 'k', long = "type")]
  pub kind: Option<String>,
  /// Key=value metadata pairs (repeatable).
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Print only the artifact ID.
  #[arg(short, long, conflicts_with = "json")]
  pub quiet: bool,
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

#[derive(Debug, Deserialize)]
struct BatchArtifactInput {
  title: String,
  #[serde(default)]
  body: Option<String>,
  #[serde(default)]
  iteration: Option<String>,
  #[serde(default)]
  metadata: std::collections::HashMap<String, String>,
  #[serde(default)]
  tags: Vec<String>,
  #[serde(default, rename = "type")]
  kind: Option<String>,
}

impl Command {
  /// Build a `NewArtifact`, persist it, and print the creation summary.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    if self.batch {
      return self.batch_call(ctx);
    }

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
      extract_title(&body).ok_or_else(|| {
        cli::Error::InvalidInput("No title found: body has no `# ` heading and no --title provided".into())
      })?
    };

    let new = NewArtifact {
      body,
      kind: self.kind.clone(),
      metadata,
      tags,
      title,
    };

    let artifact = store::create_artifact(config, new)?;

    // Process --iteration flag
    if let Some(ref iter_prefix) = self.iteration {
      let iter_id = store::resolve_iteration_id(config, iter_prefix, false)?;
      let artifact_ref = format!("artifacts/{}", artifact.id);
      store::add_iteration_task(config, &iter_id, &artifact_ref)?;
    }

    if self.json {
      let json = serde_json::to_string_pretty(&artifact)?;
      println!("{json}");
      return Ok(());
    }

    if self.quiet {
      println!("{}", artifact.id);
      return Ok(());
    }

    let id_str = artifact.id.to_string();
    let mut view = ArtifactCreateView::new(&id_str, &artifact.title, theme);
    if let Some(ref src) = self.source {
      view = view.source(src);
    }
    println!("{view}");
    Ok(())
  }

  fn batch_call(&self, ctx: &AppContext) -> cli::Result<()> {
    let config = &ctx.settings;
    let stdin = std::io::stdin();

    if stdin.is_terminal() {
      return Err(cli::Error::InvalidInput("--batch requires piped stdin".into()));
    }

    for (line_num, line) in stdin.lock().lines().enumerate() {
      let line = line.map_err(|e| cli::Error::InvalidInput(format!("line {}: {e}", line_num + 1)))?;
      if line.trim().is_empty() {
        continue;
      }

      let input: BatchArtifactInput =
        serde_json::from_str(&line).map_err(|e| cli::Error::InvalidInput(format!("line {}: {e}", line_num + 1)))?;

      let mut metadata = yaml_serde::Mapping::new();
      for (k, v) in &input.metadata {
        metadata.insert(
          yaml_serde::Value::String(k.clone()),
          yaml_serde::Value::String(v.clone()),
        );
      }

      let new = NewArtifact {
        body: input.body.unwrap_or_default(),
        kind: input.kind,
        metadata,
        tags: input.tags,
        title: input.title,
      };

      let artifact = store::create_artifact(config, new)?;

      if let Some(ref iter_prefix) = input.iteration {
        let iter_id = store::resolve_iteration_id(config, iter_prefix, false)?;
        let artifact_ref = format!("artifacts/{}", artifact.id);
        store::add_iteration_task(config, &iter_id, &artifact_ref)?;
      }

      if self.quiet {
        println!("{}", artifact.id);
      } else {
        let json = serde_json::to_string(&artifact)?;
        println!("{json}");
      }
    }

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
        batch: false,
        body: None,
        iteration: None,
        json: false,
        kind: None,
        metadata: vec![],
        quiet: false,
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
        batch: false,
        body: Some("# Content\n\nSome body text.".to_string()),
        iteration: None,
        json: false,
        kind: Some("spec".to_string()),
        metadata: vec!["version=1".to_string()],
        quiet: false,
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
        batch: false,
        body: None,
        iteration: None,
        json: false,
        kind: None,
        metadata: vec![],
        quiet: false,
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
        batch: false,
        body: Some("No heading here".to_string()),
        iteration: None,
        json: false,
        kind: None,
        metadata: vec![],
        quiet: false,
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
        batch: false,
        body: Some("# Auto Title\n\nBody text.".to_string()),
        iteration: None,
        json: false,
        kind: None,
        metadata: vec![],
        quiet: false,
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
