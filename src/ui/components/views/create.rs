//! Action confirmation view for entity creation.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders a creation confirmation with entity-specific fields.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
  /// Create a new creation confirmation for the given entity type and ID.
  pub fn new(entity_type: &str, id: &str) -> Self {
    Self {
      inner: SuccessMessage::new(format!("created {entity_type}")).id(id),
    }
  }

  /// Add a detail field to the confirmation.
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
