use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{id::Id, label::Label, value::Value},
  theme::Theme,
};

/// Renders a success message with a checkmark icon, optional entity ID, and optional key-value fields.
pub struct SuccessMessage<'a> {
  action: String,
  fields: Vec<Field>,
  id: Option<&'a str>,
  theme: &'a Theme,
}

impl<'a> SuccessMessage<'a> {
  pub fn new(action: impl Into<String>, theme: &'a Theme) -> Self {
    Self {
      action: action.into(),
      id: None,
      fields: Vec::new(),
      theme,
    }
  }

  /// Appends a label-value pair to render below the action line.
  pub fn field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
    self.fields.push(Field {
      label: label.into(),
      styled: false,
      value: value.into(),
    });
    self
  }

  /// Sets the entity ID to display after the action label.
  pub fn id(mut self, id: &'a str) -> Self {
    self.id = Some(id);
    self
  }

  /// Appends a pre-styled label-value pair that bypasses default value styling.
  pub fn styled_field(mut self, label: impl Into<String>, value: impl Display) -> Self {
    self.fields.push(Field {
      label: label.into(),
      styled: true,
      value: value.to_string(),
    });
    self
  }
}

/// A label-value pair in a success message, optionally pre-styled.
struct Field {
  label: String,
  styled: bool,
  value: String,
}

impl Display for SuccessMessage<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "  {}  {}",
      "\u{2713}".paint(self.theme.message_success_icon),
      self.action.paint(self.theme.message_created_label),
    )?;

    if let Some(id_val) = self.id {
      write!(f, "  {}", Id::new(id_val, self.theme))?;
    }

    if !self.fields.is_empty() {
      writeln!(f)?;
      let max_label = self.fields.iter().map(|f| f.label.len()).max().unwrap_or(0);
      for field in &self.fields {
        let label = Label::new(field.label.as_str(), self.theme.task_detail_label).pad_to(max_label);
        if field.styled {
          write!(f, "\n  {label}  {}", field.value)?;
        } else {
          write!(
            f,
            "\n  {label}  {}",
            Value::new(field.value.as_str(), self.theme.task_detail_value),
          )?;
        }
      }
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

  #[test]
  fn it_pads_field_labels_for_alignment() {
    let theme = theme();
    let msg = SuccessMessage::new("created artifact", &theme)
      .field("title", "probe-schema-v2")
      .field("source", "probe-schema-v2.md");
    let rendered = format!("{msg}");

    assert!(rendered.contains("title"));
    assert!(rendered.contains("probe-schema-v2"));
    assert!(rendered.contains("source"));
    assert!(rendered.contains("probe-schema-v2.md"));

    let lines: Vec<&str> = rendered.lines().collect();
    assert!(lines.len() >= 4, "expected at least 4 lines, got {}", lines.len());
  }

  #[test]
  fn it_renders_action_only() {
    let theme = theme();
    let msg = SuccessMessage::new("initialized gest in current directory", &theme);
    let rendered = format!("{msg}");

    assert!(rendered.contains('\u{2713}'));
    assert!(rendered.contains("initialized gest in current directory"));
    assert!(!rendered.contains('\n'));
  }

  #[test]
  fn it_renders_action_with_id_and_fields() {
    let theme = theme();
    let msg = SuccessMessage::new("created task", &theme)
      .id("nfkbqmrx")
      .field("title", "openai streaming adapter")
      .field("status", "\u{25CB} open");
    let rendered = format!("{msg}");

    assert!(rendered.contains('\u{2713}'));
    assert!(rendered.contains("created task"));
    assert!(rendered.contains("nf"));
    assert!(rendered.contains("kbqmrx"));
    assert!(rendered.contains("title"));
    assert!(rendered.contains("openai streaming adapter"));
    assert!(rendered.contains("status"));
  }

  #[test]
  fn it_renders_with_fields_and_no_id() {
    let theme = theme();
    let msg = SuccessMessage::new("initialized gest in current directory", &theme)
      .field("data dir", ".gest/")
      .field("config", ".gest/config.toml");
    let rendered = format!("{msg}");

    assert!(rendered.contains("data dir"));
    assert!(rendered.contains(".gest/"));
    assert!(rendered.contains("config"));
    assert!(rendered.contains(".gest/config.toml"));
  }
}
