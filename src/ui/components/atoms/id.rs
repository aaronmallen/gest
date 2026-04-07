//! Two-tone entity identifier display for terminal output.

use std::{
  collections::HashSet,
  fmt::{self, Display, Formatter},
};

use yansi::Paint;

/// Displays a truncated, two-tone entity identifier (highlighted prefix + dimmed suffix).
///
/// At most 8 characters are shown. The first `prefix_len` characters are rendered
/// in the [`id.prefix`](crate::ui::style::Theme::id_prefix) style and the remainder
/// in [`id.rest`](crate::ui::style::Theme::id_rest).
///
/// Use [`min_unique_prefix`] to compute the shortest prefix that uniquely
/// identifies each ID in a set.
pub struct Component<'a> {
  prefix_len: usize,
  value: &'a str,
}

impl<'a> Component<'a> {
  /// Create an id display, showing at most 8 characters with a 2-char highlighted prefix.
  pub fn new(value: &'a str) -> Self {
    Self {
      prefix_len: 2,
      value,
    }
  }

  /// Set the number of highlighted prefix characters.
  pub fn prefix_len(mut self, len: usize) -> Self {
    self.prefix_len = len;
    self
  }
}

impl Display for Component<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    let display: String = self.value.chars().take(8).collect();
    let prefix_len = self.prefix_len.min(display.len());
    let (prefix, rest) = display.split_at(prefix_len);

    write!(
      f,
      "{}{}",
      prefix.paint(*theme.id_prefix()),
      rest.paint(*theme.id_rest()),
    )
  }
}

/// Compute the minimum prefix length that uniquely identifies every short ID
/// in the set (each ID is first truncated to 8 chars).
///
/// Returns at least 1 and at most 8.
pub fn min_unique_prefix(ids: &[&str]) -> usize {
  let shorts: Vec<String> = ids.iter().map(|id| id.chars().take(8).collect()).collect();

  for len in 1..=8 {
    let prefixes: Vec<&str> = shorts.iter().map(|s| &s[..len.min(s.len())]).collect();
    let unique: HashSet<&str> = prefixes.iter().copied().collect();
    if unique.len() == shorts.len() {
      return len;
    }
  }
  8
}

#[cfg(test)]
mod tests {
  use super::*;

  mod min_unique_prefix_fn {
    use super::*;

    #[test]
    fn it_returns_1_when_all_ids_differ_at_first_char() {
      let ids = vec!["abcd1234", "bcde2345", "cdef3456"];

      assert_eq!(min_unique_prefix(&ids), 1);
    }

    #[test]
    fn it_increases_when_ids_share_a_prefix() {
      let ids = vec!["aaxyz111", "aaxyz222", "bbcde333"];

      // "a"/"b" collide at len=1, "aa" collides at len=2..5, unique at len=6 (aaxyz1 vs aaxyz2)
      assert_eq!(min_unique_prefix(&ids), 6);
    }

    #[test]
    fn it_returns_1_for_a_single_id() {
      let ids = vec!["abcdefgh"];

      assert_eq!(min_unique_prefix(&ids), 1);
    }

    #[test]
    fn it_returns_1_for_empty_list() {
      let ids: Vec<&str> = vec![];

      assert_eq!(min_unique_prefix(&ids), 1);
    }

    #[test]
    fn it_handles_identical_ids() {
      let ids = vec!["aaaaaaaa", "aaaaaaaa"];

      assert_eq!(min_unique_prefix(&ids), 8);
    }
  }

  mod fmt {
    use super::*;

    #[test]
    fn it_truncates_to_8_characters() {
      let id = Component::new("abcdefghijklmnop");
      let rendered = id.to_string();

      assert!(rendered.contains("ab"));
      assert!(rendered.contains("cdefgh"));
      assert!(!rendered.contains("ijklmnop"));
    }

    #[test]
    fn it_renders_short_id_without_padding() {
      let id = Component::new("abc");
      let rendered = id.to_string();

      assert!(rendered.contains("ab"));
      assert!(rendered.contains("c"));
      assert!(!rendered.contains(' '));
    }

    #[test]
    fn it_respects_custom_prefix_len() {
      yansi::disable();
      let id = Component::new("abcdefgh").prefix_len(4);
      let rendered = id.to_string();
      yansi::enable();

      // All 8 chars should be present
      assert_eq!(rendered, "abcdefgh");
    }
  }
}
