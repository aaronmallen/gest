use std::io::{self, IsTerminal, Read};

use clap::Args;

use crate::{
  config,
  config::Config,
  model::NewArtifact,
  store,
  ui::{components::Confirmation, theme::Theme},
};

/// Create a new artifact from text, a file, an editor, or stdin
#[derive(Debug, Args)]
pub struct Command {
  /// Body content as an inline string (skips editor and stdin)
  #[arg(short, long)]
  pub body: Option<String>,
  /// Read body content from a file path
  #[arg(short, long)]
  pub file: Option<String>,
  /// Artifact type (e.g. spec, adr, rfc, note)
  #[arg(long = "type")]
  pub kind: Option<String>,
  /// Key=value metadata pair (repeatable, e.g. -m key=value)
  #[arg(short, long)]
  pub metadata: Vec<String>,
  /// Comma-separated list of tags
  #[arg(long)]
  pub tags: Option<String>,
  /// Artifact title (auto-extracted from first # heading if omitted)
  #[arg(short, long)]
  pub title: Option<String>,
}

impl Command {
  pub fn call(&self, config: &Config, theme: &Theme) -> crate::Result<()> {
    let body = self.read_body()?;

    let title = if let Some(ref t) = self.title {
      t.clone()
    } else {
      extract_title(&body)
        .ok_or_else(|| crate::Error::generic("No title found: body has no `# ` heading and no --title provided"))?
    };

    let tags = self
      .tags
      .as_deref()
      .map(crate::cli::helpers::parse_tags)
      .unwrap_or_default();

    let metadata = {
      let pairs = crate::cli::helpers::split_key_value_pairs(&self.metadata)?;
      let mut map = yaml_serde::Mapping::new();
      for (key, value) in pairs {
        map.insert(yaml_serde::Value::String(key), yaml_serde::Value::String(value));
      }
      map
    };

    let new = NewArtifact {
      body,
      kind: self.kind.clone(),
      metadata,
      tags,
      title,
    };

    let data_dir = config::data_dir(config)?;
    let artifact = store::create_artifact(&data_dir, new)?;
    Confirmation::new("Created", "artifact", &artifact.id).write_to(&mut std::io::stdout(), theme)?;
    Ok(())
  }

  fn read_body(&self) -> crate::Result<String> {
    if let Some(ref body) = self.body {
      return Ok(body.clone());
    }

    if let Some(ref path) = self.file {
      let content = std::fs::read_to_string(path)?;
      return Ok(content);
    }

    if io::stdin().is_terminal() {
      if let Some(_editor) = crate::cli::editor::resolve_editor() {
        let content = crate::cli::editor::edit_temp(None, ".md")?;
        if content.trim().is_empty() {
          return Err(crate::Error::generic("Aborting: empty body"));
        }
        return Ok(content);
      }

      eprintln!("Reading body from stdin, press Ctrl+D when done...");
    }

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
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
