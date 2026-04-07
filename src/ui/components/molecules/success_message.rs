//! Success message composed from [`Id`] and [`FieldList`] components.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use super::FieldList;
use crate::ui::components::atoms::Id;

/// Renders a success message with a checkmark icon, action text, optional entity
/// ID, and optional key-value fields.
///
/// The checkmark uses the [`success.icon`](crate::ui::style::Theme::success_icon)
/// style and the action text uses
/// [`success.message`](crate::ui::style::Theme::success_message).
pub struct Component {
  action: String,
  fields: FieldList,
  has_fields: bool,
  id: Option<String>,
  prefix_len: usize,
}

impl Component {
  /// Create a success message with the given action text.
  pub fn new(action: impl Into<String>) -> Self {
    Self {
      action: action.into(),
      fields: FieldList::new(),
      has_fields: false,
      id: None,
      prefix_len: 2,
    }
  }

  /// Append a label-value pair below the action line.
  pub fn field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
    self.fields = self.fields.field(label, value);
    self.has_fields = true;
    self
  }

  /// Set the entity ID to display after the action text.
  pub fn id(mut self, id: impl Into<String>) -> Self {
    self.id = Some(id.into());
    self
  }

  /// Set the highlighted prefix length passed to the rendered [`Id`].
  pub fn prefix_len(mut self, len: usize) -> Self {
    self.prefix_len = len;
    self
  }

  /// Append a pre-styled label-value pair below the action line.
  pub fn styled_field(mut self, label: impl Into<String>, value: impl Display) -> Self {
    self.fields = self.fields.styled_field(label, value);
    self.has_fields = true;
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();

    write!(
      f,
      "  {}  {}",
      "✓".paint(*theme.message_success_icon()),
      self.action.paint(*theme.success()),
    )?;

    if let Some(id) = &self.id {
      write!(f, "  {}", Id::new(id).prefix_len(self.prefix_len))?;
    }

    if self.has_fields {
      write!(f, "\n\n{}", self.fields)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod fmt {
    use super::*;

    #[test]
    fn it_renders_action_only() {
      let component = Component::new("initialized gest");

      let output = component.to_string();

      assert!(output.contains('✓'));
      assert!(output.contains("initialized gest"));
      assert!(!output.contains('\n'));
    }

    #[test]
    fn it_renders_action_with_id() {
      let component = Component::new("created project").id("xqnuktro");

      let output = component.to_string();

      assert!(output.contains('✓'));
      assert!(output.contains("created project"));
      assert!(output.contains("xq"));
      assert!(output.contains("nuktro"));
    }

    #[test]
    fn it_renders_action_with_fields() {
      let component = Component::new("initialized gest")
        .field("data dir", ".gest/")
        .field("config", ".gest/config.toml");

      let output = component.to_string();

      assert!(output.contains('✓'));
      assert!(output.contains("initialized gest"));
      assert!(output.contains("data dir"));
      assert!(output.contains(".gest/"));
      assert!(output.contains("config"));
      assert!(output.contains(".gest/config.toml"));
    }

    #[test]
    fn it_renders_action_with_id_and_fields() {
      let component = Component::new("created task")
        .id("nfkbqmrx")
        .field("title", "openai streaming adapter")
        .field("status", "open");

      let output = component.to_string();

      assert!(output.contains('✓'));
      assert!(output.contains("created task"));
      assert!(output.contains("nf"));
      assert!(output.contains("kbqmrx"));
      assert!(output.contains("title"));
      assert!(output.contains("openai streaming adapter"));
      assert!(output.contains("status"));
      assert!(output.contains("open"));
    }

    #[test]
    fn it_renders_with_styled_field() {
      let component = Component::new("created task").styled_field("status", "\u{25CB} open");

      let output = component.to_string();

      assert!(output.contains("status"));
      assert!(output.contains("\u{25CB} open"));
    }
  }
}
