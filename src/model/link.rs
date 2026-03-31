use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Link {
  #[serde(rename = "ref")]
  pub ref_: String,
  pub rel: RelationshipType,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, ValueEnum)]
pub enum RelationshipType {
  #[serde(rename = "blocked-by")]
  BlockedBy,
  #[serde(rename = "blocks")]
  Blocks,
  #[serde(rename = "child-of")]
  ChildOf,
  #[serde(rename = "parent-of")]
  ParentOf,
  #[serde(rename = "relates-to")]
  RelatesTo,
}

impl RelationshipType {
  pub fn inverse(&self) -> Self {
    match self {
      Self::Blocks => Self::BlockedBy,
      Self::BlockedBy => Self::Blocks,
      Self::ChildOf => Self::ParentOf,
      Self::ParentOf => Self::ChildOf,
      Self::RelatesTo => Self::RelatesTo,
    }
  }
}

impl Display for RelationshipType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let s = match self {
      Self::Blocks => "blocks",
      Self::BlockedBy => "blocked-by",
      Self::ChildOf => "child-of",
      Self::ParentOf => "parent-of",
      Self::RelatesTo => "relates-to",
    };
    f.write_str(s)
  }
}

impl FromStr for RelationshipType {
  type Err = String;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "blocks" => Ok(Self::Blocks),
      "blocked-by" => Ok(Self::BlockedBy),
      "child-of" => Ok(Self::ChildOf),
      "parent-of" => Ok(Self::ParentOf),
      "relates-to" => Ok(Self::RelatesTo),
      other => Err(format!("unknown relationship type: {other}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod roundtrip {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_serializes_ref_field_correctly() {
      let link = Link {
        ref_: "https://example.com".to_string(),
        rel: RelationshipType::RelatesTo,
      };
      let toml_str = toml::to_string(&link).unwrap();
      assert!(
        toml_str.contains("ref = "),
        "Expected 'ref =' in TOML output, got: {toml_str}"
      );
      assert!(
        !toml_str.contains("ref_"),
        "Should not contain 'ref_' in TOML output, got: {toml_str}"
      );

      let roundtripped: Link = toml::from_str(&toml_str).unwrap();
      assert_eq!(link, roundtripped);
    }
  }

  mod relationship_type {
    use super::*;

    mod display {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_formats_as_kebab_case() {
        assert_eq!(RelationshipType::Blocks.to_string(), "blocks");
        assert_eq!(RelationshipType::BlockedBy.to_string(), "blocked-by");
        assert_eq!(RelationshipType::ChildOf.to_string(), "child-of");
        assert_eq!(RelationshipType::ParentOf.to_string(), "parent-of");
        assert_eq!(RelationshipType::RelatesTo.to_string(), "relates-to");
      }
    }

    mod from_str {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_parses_valid_types() {
        assert_eq!("blocks".parse::<RelationshipType>().unwrap(), RelationshipType::Blocks);
        assert_eq!(
          "blocked-by".parse::<RelationshipType>().unwrap(),
          RelationshipType::BlockedBy
        );
        assert_eq!(
          "child-of".parse::<RelationshipType>().unwrap(),
          RelationshipType::ChildOf
        );
        assert_eq!(
          "parent-of".parse::<RelationshipType>().unwrap(),
          RelationshipType::ParentOf
        );
        assert_eq!(
          "relates-to".parse::<RelationshipType>().unwrap(),
          RelationshipType::RelatesTo
        );
      }

      #[test]
      fn it_rejects_unknown() {
        assert!("invalid".parse::<RelationshipType>().is_err());
      }
    }

    mod inverse {
      use pretty_assertions::assert_eq;

      use super::*;

      #[test]
      fn it_returns_correct_inverse() {
        assert_eq!(RelationshipType::Blocks.inverse(), RelationshipType::BlockedBy);
        assert_eq!(RelationshipType::BlockedBy.inverse(), RelationshipType::Blocks);
        assert_eq!(RelationshipType::ChildOf.inverse(), RelationshipType::ParentOf);
        assert_eq!(RelationshipType::ParentOf.inverse(), RelationshipType::ChildOf);
        assert_eq!(RelationshipType::RelatesTo.inverse(), RelationshipType::RelatesTo);
      }
    }
  }
}
