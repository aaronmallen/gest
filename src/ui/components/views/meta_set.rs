//! Action confirmation view for metadata set.

use std::fmt::{self, Display, Formatter};

use crate::ui::components::molecules::SuccessMessage;

/// Renders a metadata set confirmation.
pub struct Component {
  inner: SuccessMessage,
}

impl Component {
  pub fn new(entity_type: &str, id: &str, key: &str, value: &str) -> Self {
    Self {
      inner: SuccessMessage::new(format!("updated {entity_type}"))
        .id(id)
        .field("key", key)
        .field("value", value),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.inner)
  }
}
