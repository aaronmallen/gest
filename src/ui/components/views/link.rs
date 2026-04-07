//! Action confirmation view for entity linking.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders a link confirmation with link details.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
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
