//! Action confirmation view for entity updates.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders an update confirmation with changed fields.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
  pub fn new(entity_type: &str, id: &str) -> Self {
    Self {
      inner: SuccessMessage::new(format!("updated {entity_type}")).id(id),
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
