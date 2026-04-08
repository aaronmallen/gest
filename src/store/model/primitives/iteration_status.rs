use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The lifecycle status of an iteration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Primitive {
  /// In-progress iteration accepting work (the default).
  #[default]
  Active,
  /// Terminal: iteration was abandoned before completion.
  Cancelled,
  /// Terminal: iteration finished successfully.
  Completed,
}

impl Primitive {
  /// Whether this status represents a terminal (final) state.
  pub fn is_terminal(self) -> bool {
    matches!(self, Self::Cancelled | Self::Completed)
  }
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Active => f.write_str("active"),
      Self::Cancelled => f.write_str("cancelled"),
      Self::Completed => f.write_str("completed"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "active" => Ok(Self::Active),
      "cancelled" => Ok(Self::Cancelled),
      "completed" => Ok(Self::Completed),
      other => Err(format!("invalid iteration status: {other}")),
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
    fn it_formats_active() {
      assert_eq!(Primitive::Active.to_string(), "active");
    }

    #[test]
    fn it_formats_cancelled() {
      assert_eq!(Primitive::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn it_formats_completed() {
      assert_eq!(Primitive::Completed.to_string(), "completed");
    }
  }

  mod is_terminal {
    use super::*;

    #[test]
    fn it_returns_false_for_active() {
      assert!(!Primitive::Active.is_terminal());
    }

    #[test]
    fn it_returns_true_for_cancelled() {
      assert!(Primitive::Cancelled.is_terminal());
    }

    #[test]
    fn it_returns_true_for_completed() {
      assert!(Primitive::Completed.is_terminal());
    }
  }
}
