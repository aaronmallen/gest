use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The kind of event recorded in the audit trail.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Primitive {
  #[serde(rename = "phase-change")]
  Phase,
  #[serde(rename = "priority-change")]
  Priority,
  #[serde(rename = "status-change")]
  Status,
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Phase => f.write_str("phase-change"),
      Self::Priority => f.write_str("priority-change"),
      Self::Status => f.write_str("status-change"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "phase-change" => Ok(Self::Phase),
      "priority-change" => Ok(Self::Priority),
      "status-change" => Ok(Self::Status),
      other => Err(format!("invalid event kind: {other}")),
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
    fn it_formats_phase() {
      assert_eq!(Primitive::Phase.to_string(), "phase-change");
    }

    #[test]
    fn it_formats_priority() {
      assert_eq!(Primitive::Priority.to_string(), "priority-change");
    }

    #[test]
    fn it_formats_status() {
      assert_eq!(Primitive::Status.to_string(), "status-change");
    }
  }

  mod from_str {
    use super::*;

    #[test]
    fn it_parses_kebab_case() {
      assert_eq!("phase-change".parse::<Primitive>().unwrap(), Primitive::Phase);
    }

    #[test]
    fn it_rejects_invalid() {
      assert!("created".parse::<Primitive>().is_err());
    }
  }
}
