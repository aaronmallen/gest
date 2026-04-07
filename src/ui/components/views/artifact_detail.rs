use std::fmt::{self, Display, Formatter};

use super::super::atoms::{Id, Label, Separator, Tag, Value};
use crate::ui::{components::molecules::row, markdown, style};

/// Indentation prefix for detail rows.
const INDENT: &str = "  ";

/// Fixed padding width for field labels.
const LABEL_PAD: usize = 9;

/// Renders the full detail view for a single artifact, including metadata and optional body content.
pub struct Component {
  body: Option<String>,
  created: Option<String>,
  id: String,
  notes: Vec<NoteView>,
  tags: Vec<String>,
  title: String,
  updated: Option<String>,
}

struct NoteView {
  body: String,
  id: String,
}

impl Component {
  pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
    Self {
      body: None,
      created: None,
      id: id.into(),
      notes: Vec::new(),
      tags: Vec::new(),
      title: title.into(),
      updated: None,
    }
  }

  /// Marks this artifact as archived (currently unused in detail rendering but kept for API compat).
  pub fn archived(self) -> Self {
    self
  }

  /// Sets optional markdown body content to render below the metadata fields.
  pub fn body(mut self, body: impl Into<String>) -> Self {
    self.body = Some(body.into());
    self
  }

  /// Sets the created timestamp.
  pub fn created(mut self, created: impl Into<String>) -> Self {
    self.created = Some(created.into());
    self
  }

  /// Adds a note to the detail view.
  pub fn note(mut self, id: impl Into<String>, body: impl Into<String>) -> Self {
    self.notes.push(NoteView {
      body: body.into(),
      id: id.into(),
    });
    self
  }

  pub fn tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }

  /// Sets the updated timestamp.
  pub fn updated(mut self, updated: impl Into<String>) -> Self {
    self.updated = Some(updated.into());
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();
    let width = row::terminal_width() as usize;

    let label_style = *theme.artifact_detail_label();
    let value_style = *theme.artifact_detail_value();

    writeln!(f, "{}", Id::new(&self.id))?;

    writeln!(f)?;

    let label = Label::new("title", label_style).pad_to(LABEL_PAD);
    let value = Value::new(&self.title, value_style);
    writeln!(f, "{INDENT}{label}{value}")?;

    if !self.tags.is_empty() {
      let label = Label::new("tags", label_style).pad_to(LABEL_PAD);
      let tags = Tag::new(self.tags.clone(), *theme.tag());
      writeln!(f, "{INDENT}{label}{tags}")?;
    }

    if let Some(ref created) = self.created {
      let label = Label::new("created", label_style).pad_to(LABEL_PAD);
      let value = Value::new(created, value_style);
      writeln!(f, "{INDENT}{label}{value}")?;
    }

    if let Some(ref updated) = self.updated {
      let label = Label::new("updated", label_style).pad_to(LABEL_PAD);
      let value = Value::new(updated, value_style);
      writeln!(f, "{INDENT}{label}{value}")?;
    }

    if let Some(ref body) = self.body {
      writeln!(f)?;

      let sep = Separator::labeled("content", *theme.artifact_detail_separator()).width(width.saturating_sub(2));
      writeln!(f, "{INDENT}{sep}")?;
      writeln!(f)?;

      let rendered = markdown::render(body);
      for line in rendered.lines() {
        writeln!(f, "{INDENT}{line}")?;
      }

      writeln!(f)?;

      let rule = Separator::rule(*theme.artifact_detail_separator()).width(width.saturating_sub(2));
      writeln!(f, "{INDENT}{rule}")?;
    }

    if !self.notes.is_empty() {
      writeln!(f)?;
      writeln!(
        f,
        "{INDENT}{}",
        Separator::labeled("notes", *theme.artifact_detail_separator()).width(width.saturating_sub(2))
      )?;
      for note in &self.notes {
        writeln!(f)?;
        let label = Label::new("note", label_style).pad_to(LABEL_PAD);
        let id = Id::new(&note.id);
        writeln!(f, "{INDENT}{label}{id}")?;
        let label = Label::new("body", label_style).pad_to(LABEL_PAD);
        let value = Value::new(&note.body, value_style);
        writeln!(f, "{INDENT}{label}{value}")?;
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(detail: &Component) -> String {
    yansi::disable();
    let out = detail.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_inserts_blank_line_after_id() {
    let detail = Component::new("fsahdqlt", "probe-schema-v2")
      .created("2026-03-27")
      .updated("2026-03-29");
    let output = render(&detail);
    let lines: Vec<&str> = output.lines().collect();

    assert!(lines.len() >= 2 && lines[1].is_empty(), "second line should be blank");
  }

  #[test]
  fn it_omits_content_section_when_no_body() {
    let detail = Component::new("abcdefgh", "bare-artifact");
    let output = render(&detail);

    assert!(!output.contains("content"), "should not contain content separator");
    assert!(!output.contains('\u{2500}'), "should not contain rule characters");
  }

  #[test]
  fn it_omits_tag_line_when_no_tags() {
    let detail = Component::new("abcdefgh", "bare-artifact");
    let output = render(&detail);

    assert!(!output.contains('#'), "should not contain tag marker");
  }

  #[test]
  fn it_renders_body_with_separators() {
    let detail = Component::new("fsahdqlt", "probe-schema-v2").body("## heading\n\nSome content here.");
    let output = render(&detail);

    assert!(output.contains("content"), "should contain content label in separator");
    assert!(output.contains("heading"), "should render markdown heading");
    assert!(output.contains("Some content here."), "should render markdown body");
    assert!(output.contains('\u{2500}'), "should contain rule characters");
  }

  #[test]
  fn it_renders_id_and_metadata() {
    let tags = vec!["schema".to_string()];
    let detail = Component::new("fsahdqlt", "probe-schema-v2")
      .tags(tags)
      .created("2026-03-27")
      .updated("2026-03-29");
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
    let tags = vec!["spec".to_string(), "backend".to_string()];
    let detail = Component::new("abcdefgh", "my-artifact").tags(tags);
    let output = render(&detail);

    assert!(output.contains("#spec"), "should contain first tag");
    assert!(output.contains("#backend"), "should contain second tag");
  }

  #[test]
  fn it_shows_id_on_first_line() {
    let detail = Component::new("fsahdqlt", "probe-schema-v2");
    let output = render(&detail);
    let first_line = output.lines().next().unwrap();

    assert!(first_line.contains("fsahdqlt"), "id should be on the first line");
  }

  #[test]
  fn it_renders_notes_section() {
    let detail = Component::new("fsahdqlt", "probe-schema-v2").note("abcd1234", "This is a note.");
    let output = render(&detail);

    assert!(output.contains("notes"), "should contain notes separator");
    assert!(output.contains("abcd1234"), "should contain note id");
    assert!(output.contains("This is a note."), "should contain note body");
  }
}
