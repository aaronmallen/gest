//! Shared `--priority` value parser for the `task create` and `task update` commands.
//!
//! Accepts either the labelled form (`critical|high|medium|low|lowest`, case-insensitive)
//! or the legacy integer form (`0..=4`). Returns the integer representation so that
//! downstream call sites and the storage schema are unchanged.

use std::str::FromStr;

use crate::store::model::primitives::Priority;

/// Human-readable list of accepted forms, used in error messages and doc strings.
pub(super) const ACCEPTED_FORMS: &str = "0-4, critical, high, medium, low, lowest";

/// Parse a `--priority` argument value.
///
/// Tries the labelled form first, then falls back to the integer form. Both are
/// validated against [`Priority`]; the parsed value is returned as its `u8`
/// representation.
pub(super) fn parse_priority(value: &str) -> Result<u8, String> {
  if let Ok(priority) = Priority::from_str(value) {
    return Ok(priority.into());
  }

  match value.parse::<u8>() {
    Ok(byte) => Priority::try_from(byte)
      .map(u8::from)
      .map_err(|_| format!("invalid priority `{value}` (accepted: {ACCEPTED_FORMS})")),
    Err(_) => Err(format!("invalid priority `{value}` (accepted: {ACCEPTED_FORMS})")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parse_priority {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_accepts_each_integer_in_range() {
      assert_eq!(parse_priority("0").unwrap(), 0);
      assert_eq!(parse_priority("1").unwrap(), 1);
      assert_eq!(parse_priority("2").unwrap(), 2);
      assert_eq!(parse_priority("3").unwrap(), 3);
      assert_eq!(parse_priority("4").unwrap(), 4);
    }

    #[test]
    fn it_accepts_each_label() {
      assert_eq!(parse_priority("critical").unwrap(), 0);
      assert_eq!(parse_priority("high").unwrap(), 1);
      assert_eq!(parse_priority("medium").unwrap(), 2);
      assert_eq!(parse_priority("low").unwrap(), 3);
      assert_eq!(parse_priority("lowest").unwrap(), 4);
    }

    #[test]
    fn it_accepts_labels_case_insensitively() {
      assert_eq!(parse_priority("CRITICAL").unwrap(), 0);
      assert_eq!(parse_priority("High").unwrap(), 1);
      assert_eq!(parse_priority("mEdIuM").unwrap(), 2);
    }

    #[test]
    fn it_rejects_an_empty_string() {
      let err = parse_priority("").unwrap_err();
      assert!(err.contains("accepted"));
    }

    #[test]
    fn it_rejects_integers_out_of_range() {
      let err_five = parse_priority("5").unwrap_err();
      let err_max = parse_priority("255").unwrap_err();

      assert!(err_five.contains("5"));
      assert!(err_five.contains("accepted"));
      assert!(err_max.contains("255"));
    }

    #[test]
    fn it_rejects_unknown_labels() {
      let err = parse_priority("urgent").unwrap_err();

      assert!(err.contains("urgent"));
      assert!(err.contains("accepted"));
    }
  }
}
