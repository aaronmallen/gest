//! View for undo confirmation.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders an undo confirmation.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
  pub fn new(entity_type: &str, id: &str, action: &str) -> Self {
    Self {
      inner: SuccessMessage::new(format!("undone {entity_type}"))
        .id(id)
        .field("action", action),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.inner)
  }
}
