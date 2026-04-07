//! Action confirmation view for state changes (complete, cancel, reopen, claim, archive).

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders a state change confirmation.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
  /// Create a state change confirmation.
  /// `action` is the verb (e.g. "completed", "cancelled", "reopened", "claimed", "archived").
  pub fn new(action: &str, entity_type: &str, id: &str) -> Self {
    Self {
      inner: SuccessMessage::new(format!("{action} {entity_type}")).id(id),
    }
  }

  pub fn field(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
    self.inner = self.inner.field(label, value);
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.inner)
  }
}
