//! Single-project show view using a FieldList to render id and root.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  components::{atoms::Id, molecules::FieldList},
  style,
};

/// Renders the current project's id and root path as an aligned key-value list.
pub struct Component {
  id: String,
  root: String,
}

impl Component {
  pub fn new(id: impl Into<String>, root: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      root: root.into(),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();
    let id = Id::new(&self.id);
    let root = self.root.paint(*theme.project_show_value()).to_string();

    let fields = FieldList::new().styled_field("id", id).styled_field("root", root);
    write!(f, "{fields}")
  }
}
