//! View for displaying a metadata value.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

/// Renders a styled metadata value.
pub struct Component {
  value: String,
}

impl Component {
  pub fn new(value: impl Into<String>) -> Self {
    Self {
      value: value.into(),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    write!(f, "{}", self.value.paint(*theme.meta_value()))
  }
}
