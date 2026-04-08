//! Placeholder molecule shown in place of an empty list view.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::style;

/// A styled placeholder message for empty entity lists.
pub struct Component {
  entity: &'static str,
}

impl Component {
  /// Create a new empty list placeholder.
  pub fn new(entity: &'static str) -> Self {
    Self {
      entity,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();
    write!(f, "  {}", format!("no {}", self.entity).paint(*theme.muted()))
  }
}
