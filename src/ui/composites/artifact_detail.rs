use std::fmt::{self, Display, Formatter};

use crate::ui::{
  atoms::{id::Id, label::Label, separator::Separator, tag::Tags, value::Value},
  markdown,
  theming::theme::Theme,
  utils,
};

/// Indentation prefix for detail rows.
const INDENT: &str = "  ";

/// Fixed padding width for field labels.
const LABEL_PAD: usize = 9;

/// Renders the full detail view for a single artifact, including metadata and optional body content.
pub struct ArtifactDetail<'a> {
  body: Option<&'a str>,
  created: &'a str,
  id: &'a str,
  tags: &'a [String],
  theme: &'a Theme,
  title: &'a str,
  updated: &'a str,
}

impl<'a> ArtifactDetail<'a> {
  pub fn new(
    id: &'a str,
    title: &'a str,
    tags: &'a [String],
    created: &'a str,
    updated: &'a str,
    theme: &'a Theme,
  ) -> Self {
    Self {
      id,
      title,
      tags,
      created,
      updated,
      body: None,
      theme,
    }
  }

  /// Sets optional markdown body content to render below the metadata fields.
  pub fn body(mut self, body: Option<&'a str>) -> Self {
    self.body = body;
    self
  }
}

impl Display for ArtifactDetail<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let width = utils::terminal_width() as usize;

    writeln!(f, "{}", Id::new(self.id, self.theme))?;

    writeln!(f)?;

    let label_style = self.theme.artifact_detail_label;
    let value_style = self.theme.artifact_detail_value;

    let label = Label::new("title", label_style).pad_to(LABEL_PAD);
    let value = Value::new(self.title, value_style);
    writeln!(f, "{INDENT}{label}{value}")?;

    if !self.tags.is_empty() {
      let label = Label::new("tags", label_style).pad_to(LABEL_PAD);
      let tags = Tags::new(self.tags, self.theme.tag);
      writeln!(f, "{INDENT}{label}{tags}")?;
    }

    let label = Label::new("created", label_style).pad_to(LABEL_PAD);
    let value = Value::new(self.created, value_style);
    writeln!(f, "{INDENT}{label}{value}")?;

    let label = Label::new("updated", label_style).pad_to(LABEL_PAD);
    let value = Value::new(self.updated, value_style);
    writeln!(f, "{INDENT}{label}{value}")?;

    if let Some(body) = self.body {
      writeln!(f)?;

      let sep = Separator::labeled("content", self.theme.artifact_detail_separator).width(width.saturating_sub(2));
      writeln!(f, "{INDENT}{sep}")?;
      writeln!(f)?;

      let content_width = width.saturating_sub(4);
      let rendered = markdown::render(body);
      for line in rendered.lines() {
        writeln!(f, "{INDENT}{line}")?;
      }

      writeln!(f)?;

      let rule = Separator::rule(self.theme.artifact_detail_separator).width(width.saturating_sub(2));
      writeln!(f, "{INDENT}{rule}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(detail: &ArtifactDetail) -> String {
    yansi::disable();
    let out = detail.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_inserts_blank_line_after_id() {
    let theme = theme();
    let detail = ArtifactDetail::new("fsahdqlt", "probe-schema-v2", &[], "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);
    let lines: Vec<&str> = output.lines().collect();

    assert!(lines.len() >= 2 && lines[1].is_empty(), "second line should be blank");
  }

  #[test]
  fn it_omits_content_section_when_no_body() {
    let theme = theme();
    let detail = ArtifactDetail::new("abcdefgh", "bare-artifact", &[], "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);

    assert!(!output.contains("content"), "should not contain content separator");
    assert!(!output.contains('─'), "should not contain rule characters");
  }

  #[test]
  fn it_omits_tag_line_when_no_tags() {
    let theme = theme();
    let detail = ArtifactDetail::new("abcdefgh", "bare-artifact", &[], "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);

    assert!(!output.contains('#'), "should not contain tag marker");
  }

  #[test]
  fn it_renders_body_with_separators() {
    let theme = theme();
    let detail = ArtifactDetail::new("fsahdqlt", "probe-schema-v2", &[], "2026-03-27", "2026-03-29", &theme)
      .body(Some("## heading\n\nSome content here."));
    let output = render(&detail);

    assert!(output.contains("content"), "should contain content label in separator");
    assert!(output.contains("heading"), "should render markdown heading");
    assert!(output.contains("Some content here."), "should render markdown body");
    assert!(output.contains('─'), "should contain rule characters");
  }

  #[test]
  fn it_renders_id_and_metadata() {
    let theme = theme();
    let tags = vec!["schema".to_string()];
    let detail = ArtifactDetail::new("fsahdqlt", "probe-schema-v2", &tags, "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);

    assert!(output.contains("fsahdqlt"), "should contain the id");
    assert!(output.contains("title"), "should contain title label");
    assert!(output.contains("probe-schema-v2"), "should contain title value");
    assert!(output.contains("#schema"), "should contain tags");
    assert!(output.contains("2026-03-27"), "should contain created date");
    assert!(output.contains("2026-03-29"), "should contain updated date");
  }

  #[test]
  fn it_renders_multiple_tags() {
    let theme = theme();
    let tags = vec!["spec".to_string(), "backend".to_string()];
    let detail = ArtifactDetail::new("abcdefgh", "my-artifact", &tags, "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);

    assert!(output.contains("#spec"), "should contain first tag");
    assert!(output.contains("#backend"), "should contain second tag");
  }

  #[test]
  fn it_shows_id_on_first_line() {
    let theme = theme();
    let detail = ArtifactDetail::new("fsahdqlt", "probe-schema-v2", &[], "2026-03-27", "2026-03-29", &theme);
    let output = render(&detail);
    let first_line = output.lines().next().unwrap();

    assert!(first_line.contains("fsahdqlt"), "id should be on the first line");
  }
}
