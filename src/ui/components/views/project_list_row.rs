use std::fmt::{self, Display, Formatter};

use super::super::{atoms::Id, molecules::FieldList};

/// A project summary row for list display.
pub struct Component {
  id: String,
  root: String,
}

impl Component {
  pub fn new(id: String, root: String) -> Self {
    Self {
      id,
      root,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let fields = FieldList::new()
      .styled_field("id", Id::new(&self.id))
      .field("root", self.root.clone());
    write!(f, "{fields}")
  }
}
