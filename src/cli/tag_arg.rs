//! Shared parser and normalizer for `--tag`/`tags` CLI arguments.
//!
//! CLI tag inputs are split on `,` by clap's `value_delimiter` and each piece is trimmed
//! by [`trim_tag`]. Because clap's `value_parser` cannot drop values, callers must then
//! run the collected list through [`normalize_tags`] to discard blank entries produced by
//! stray or trailing commas (e.g. `"a,,b"` or `"a,b,"`).

/// Collect tag labels while dropping empty entries produced by stray or trailing commas.
pub fn normalize_tags(raw: &[String]) -> Vec<String> {
  raw.iter().filter(|label| !label.is_empty()).cloned().collect()
}

/// Trim whitespace around a single tag value.
///
/// Used as a clap `value_parser` on tag arguments so values survive splitting on `,` with
/// surrounding whitespace preserved only inside labels.
pub fn trim_tag(s: &str) -> Result<String, String> {
  Ok(s.trim().to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  mod normalize_tags_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_drops_empty_entries() {
      let raw = vec!["a".to_string(), "".to_string(), "b".to_string()];

      assert_eq!(normalize_tags(&raw), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn it_preserves_order() {
      let raw = vec!["c".to_string(), "a".to_string(), "b".to_string()];

      assert_eq!(
        normalize_tags(&raw),
        vec!["c".to_string(), "a".to_string(), "b".to_string()]
      );
    }

    #[test]
    fn it_returns_empty_for_all_blank() {
      let raw = vec!["".to_string(), "".to_string()];

      assert!(normalize_tags(&raw).is_empty());
    }
  }

  mod trim_tag_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_empty_for_whitespace_only() {
      assert_eq!(trim_tag("   ").unwrap(), "");
    }

    #[test]
    fn it_returns_unchanged_when_no_whitespace() {
      assert_eq!(trim_tag("tag").unwrap(), "tag");
    }

    #[test]
    fn it_trims_leading_and_trailing_whitespace() {
      assert_eq!(trim_tag("  hello  ").unwrap(), "hello");
    }
  }
}
