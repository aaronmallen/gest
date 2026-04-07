use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The lifecycle status of a task.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Primitive {
  Cancelled,
  Done,
  InProgress,
  #[default]
  Open,
}

impl Primitive {
  /// Whether this status represents a terminal (final) state.
  pub fn is_terminal(self) -> bool {
    matches!(self, Self::Cancelled | Self::Done)
  }
}

impl Display for Primitive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Cancelled => f.write_str("cancelled"),
      Self::Done => f.write_str("done"),
      Self::InProgress => f.write_str("in-progress"),
      Self::Open => f.write_str("open"),
    }
  }
}

impl FromStr for Primitive {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "cancelled" => Ok(Self::Cancelled),
      "done" => Ok(Self::Done),
      "in-progress" | "in_progress" | "inprogress" => Ok(Self::InProgress),
      "open" => Ok(Self::Open),
      other => Err(format!("invalid task status: {other}")),
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
    fn it_formats_cancelled() {
      assert_eq!(Primitive::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn it_formats_done() {
      assert_eq!(Primitive::Done.to_string(), "done");
    }

    #[test]
    fn it_formats_in_progress() {
      assert_eq!(Primitive::InProgress.to_string(), "in-progress");
    }

    #[test]
    fn it_formats_open() {
      assert_eq!(Primitive::Open.to_string(), "open");
    }
  }

  mod is_terminal {
    use super::*;

    #[test]
    fn it_returns_false_for_in_progress() {
      assert!(!Primitive::InProgress.is_terminal());
    }

    #[test]
    fn it_returns_false_for_open() {
      assert!(!Primitive::Open.is_terminal());
    }

    #[test]
    fn it_returns_true_for_cancelled() {
      assert!(Primitive::Cancelled.is_terminal());
    }

    #[test]
    fn it_returns_true_for_done() {
      assert!(Primitive::Done.is_terminal());
    }
  }
}
