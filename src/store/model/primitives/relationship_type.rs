use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The type of relationship between two entities.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Primitive {
  /// Source cannot progress until the target is resolved.
  BlockedBy,
  /// Source prevents the target from progressing.
  Blocks,
  /// Source is a child of the target in a parent/child hierarchy.
  ChildOf,
  /// Source is a parent of the target in a parent/child hierarchy.
  ParentOf,
  /// Symmetric, loose association between source and target.
  RelatesTo,
}

impl Primitive {
  /// Returns the inverse relationship type.
  pub fn inverse(self) -> Self {
    match self {
      Self::BlockedBy => Self::Blocks,
      Self::Blocks => Self::BlockedBy,
      Self::ChildOf => Self::ParentOf,
      Self::ParentOf => Self::ChildOf,
      Self::RelatesTo => Self::RelatesTo,
    }
  }
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::BlockedBy => f.write_str("blocked-by"),
      Self::Blocks => f.write_str("blocks"),
      Self::ChildOf => f.write_str("child-of"),
      Self::ParentOf => f.write_str("parent-of"),
      Self::RelatesTo => f.write_str("relates-to"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "blocked-by" => Ok(Self::BlockedBy),
      "blocks" => Ok(Self::Blocks),
      "child-of" => Ok(Self::ChildOf),
      "parent-of" => Ok(Self::ParentOf),
      "relates-to" => Ok(Self::RelatesTo),
      other => Err(format!("invalid relationship type: {other}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_formats_blocked_by() {
      assert_eq!(Primitive::BlockedBy.to_string(), "blocked-by");
    }

    #[test]
    fn it_formats_blocks() {
      assert_eq!(Primitive::Blocks.to_string(), "blocks");
    }

    #[test]
    fn it_formats_child_of() {
      assert_eq!(Primitive::ChildOf.to_string(), "child-of");
    }

    #[test]
    fn it_formats_parent_of() {
      assert_eq!(Primitive::ParentOf.to_string(), "parent-of");
    }

    #[test]
    fn it_formats_relates_to() {
      assert_eq!(Primitive::RelatesTo.to_string(), "relates-to");
    }
  }

  mod inverse {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_inverts_blocked_by_to_blocks() {
      assert_eq!(Primitive::BlockedBy.inverse(), Primitive::Blocks);
    }

    #[test]
    fn it_inverts_blocks_to_blocked_by() {
      assert_eq!(Primitive::Blocks.inverse(), Primitive::BlockedBy);
    }

    #[test]
    fn it_inverts_child_of_to_parent_of() {
      assert_eq!(Primitive::ChildOf.inverse(), Primitive::ParentOf);
    }

    #[test]
    fn it_inverts_parent_of_to_child_of() {
      assert_eq!(Primitive::ParentOf.inverse(), Primitive::ChildOf);
    }

    #[test]
    fn it_keeps_relates_to_symmetric() {
      assert_eq!(Primitive::RelatesTo.inverse(), Primitive::RelatesTo);
    }
  }
}
