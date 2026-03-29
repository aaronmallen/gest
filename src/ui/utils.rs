use yansi::Paint;

use super::theme::Theme;
use crate::model::{Id, Status, iteration};

/// Returns the display width of a string after stripping ANSI escape sequences.
///
/// This is useful for column alignment when strings may contain ANSI color codes.
pub fn display_width(s: &str) -> usize {
  let bytes = s.as_bytes();
  let mut width = 0usize;
  let mut i = 0;

  while i < bytes.len() {
    if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
      // Skip the ESC [ ... m sequence
      i += 2;
      while i < bytes.len() && bytes[i] != b'm' {
        i += 1;
      }
      if i < bytes.len() {
        i += 1; // skip 'm'
      }
    } else {
      width += 1;
      i += 1;
    }
  }

  width
}

pub fn format_id(id: &Id, prefix_len: usize, max_display_len: Option<usize>, theme: &Theme) -> String {
  let full = id.to_string();
  let s = match max_display_len {
    Some(n) => &full[..n.min(full.len())],
    None => &full,
  };
  let (prefix, rest) = s.split_at(prefix_len.min(s.len()));
  format!("{}{}", prefix.paint(theme.id_prefix), rest.paint(theme.id_rest))
}

pub fn format_iteration_status(status: &iteration::Status, theme: &Theme) -> String {
  match status {
    iteration::Status::Active => status.to_string().paint(theme.status_in_progress).to_string(),
    iteration::Status::Completed => status.to_string().paint(theme.status_done).to_string(),
    iteration::Status::Failed => status.to_string().paint(theme.status_cancelled).to_string(),
  }
}

pub fn format_status(status: &Status, theme: &Theme) -> String {
  match status {
    Status::Open => status.to_string().paint(theme.status_open).to_string(),
    Status::InProgress => status.to_string().paint(theme.status_in_progress).to_string(),
    Status::Done => status.to_string().paint(theme.status_done).to_string(),
    Status::Cancelled => status.to_string().paint(theme.status_cancelled).to_string(),
  }
}

pub fn format_tags(tags: &[String], theme: &Theme) -> String {
  tags
    .iter()
    .map(|t| format!("@{}", t).paint(theme.tag).to_string())
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn shortest_unique_prefixes(ids: &[String]) -> Vec<usize> {
  // Trie node: count of IDs passing through this node, plus children indexed by byte value.
  // Since IDs use only chars k-z (16 chars), we use a HashMap for children.
  struct TrieNode {
    children: [Option<Box<TrieNode>>; 16],
    count: usize,
  }

  impl TrieNode {
    fn new() -> Self {
      const NONE: Option<Box<TrieNode>> = None;
      Self {
        children: [NONE; 16],
        count: 0,
      }
    }
  }

  // Map chars k-z to indices 0-15
  fn char_index(c: u8) -> usize {
    (c - b'k') as usize
  }

  // Build trie
  let mut root = TrieNode::new();
  for id in ids {
    let mut node = &mut root;
    for &b in id.as_bytes() {
      let idx = char_index(b);
      node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::new()));
      node.count += 1;
    }
  }

  // For each ID, walk the trie to find shortest unique prefix
  ids
    .iter()
    .map(|id| {
      let mut node = &root;
      let mut len = 0;
      for &b in id.as_bytes() {
        let idx = char_index(b);
        node = node.children[idx].as_ref().unwrap();
        len += 1;
        if node.count == 1 {
          break;
        }
      }
      len
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  mod display_width {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_zero_for_empty_string() {
      assert_eq!(display_width(""), 0);
    }

    #[test]
    fn it_returns_length_for_plain_text() {
      assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn it_strips_single_ansi_sequence() {
      // "\x1b[31m" = red, "\x1b[0m" = reset
      assert_eq!(display_width("\x1b[31mhello\x1b[0m"), 5);
    }

    #[test]
    fn it_strips_multiple_ansi_sequences() {
      // bold red text followed by reset, then bold blue text followed by reset
      let s = "\x1b[1;31mhello\x1b[0m \x1b[1;34mworld\x1b[0m";
      assert_eq!(display_width(s), 11); // "hello world"
    }

    #[test]
    fn it_handles_ansi_only_string() {
      assert_eq!(display_width("\x1b[31m\x1b[0m"), 0);
    }

    #[test]
    fn it_handles_text_without_reset() {
      assert_eq!(display_width("\x1b[1;4;35mheading"), 7);
    }
  }

  mod format_id {
    use super::*;

    #[test]
    fn it_clamps_prefix_to_id_length() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let theme = Theme::default();
      let formatted = format_id(&id, 100, None, &theme);
      assert!(
        formatted.contains("zyxwvutsrqponmlkzyxwvutsrqponmlk"),
        "Should contain full ID when prefix exceeds length"
      );
    }

    #[test]
    fn it_contains_both_prefix_and_remainder() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let theme = Theme::default();
      let formatted = format_id(&id, 3, None, &theme);
      assert!(formatted.contains("zyx"), "Should contain the prefix text");
      assert!(
        formatted.contains("wvutsrqponmlkzyxwvutsrqponmlk"),
        "Should contain the remainder text"
      );
    }

    #[test]
    fn it_handles_exact_prefix_length() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let theme = Theme::default();
      let formatted = format_id(&id, 32, None, &theme);
      assert!(
        formatted.contains("zyxwvutsrqponmlkzyxwvutsrqponmlk"),
        "Should contain full ID when prefix is full length"
      );
    }

    #[test]
    fn it_handles_prefix_length_of_zero() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let theme = Theme::default();
      let formatted = format_id(&id, 0, None, &theme);
      assert!(
        formatted.contains("zyxwvutsrqponmlkzyxwvutsrqponmlk"),
        "Should contain full ID with zero prefix"
      );
    }

    #[test]
    fn it_truncates_to_max_display_len() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      let theme = Theme::default();
      let formatted = format_id(&id, 3, Some(8), &theme);
      assert!(formatted.contains("zyx"), "Should contain the prefix text");
      assert!(formatted.contains("wvuts"), "Should contain truncated remainder");
      assert!(
        !formatted.contains("rqponmlk"),
        "Should not contain chars beyond max_display_len"
      );
    }
  }

  mod format_status {
    use super::*;

    #[test]
    fn it_contains_status_text_for_cancelled() {
      let theme = Theme::default();
      let result = format_status(&Status::Cancelled, &theme);
      assert!(result.contains("cancelled"), "Should contain 'cancelled'");
    }

    #[test]
    fn it_contains_status_text_for_done() {
      let theme = Theme::default();
      let result = format_status(&Status::Done, &theme);
      assert!(result.contains("done"), "Should contain 'done'");
    }

    #[test]
    fn it_contains_status_text_for_in_progress() {
      let theme = Theme::default();
      let result = format_status(&Status::InProgress, &theme);
      assert!(result.contains("in-progress"), "Should contain 'in-progress'");
    }

    #[test]
    fn it_contains_status_text_for_open() {
      let theme = Theme::default();
      let result = format_status(&Status::Open, &theme);
      assert!(result.contains("open"), "Should contain 'open'");
    }
  }

  mod shortest_unique_prefixes {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_differentiates_shared_prefix() {
      let ids = vec![
        "zyxwvutsrqponmlkzyxwvutsrqponmlk".to_string(),
        "zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
      ];
      assert_eq!(shortest_unique_prefixes(&ids), vec![5, 5]);
    }

    #[test]
    fn it_handles_empty_list() {
      let ids: Vec<String> = vec![];
      assert!(shortest_unique_prefixes(&ids).is_empty());
    }

    #[test]
    fn it_handles_no_overlap() {
      let ids = vec![
        "zyxwvutsrqponmlkzyxwvutsrqponmlk".to_string(),
        "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
      ];
      assert_eq!(shortest_unique_prefixes(&ids), vec![1, 1]);
    }

    #[test]
    fn it_handles_three_ids_with_mixed_overlap() {
      let ids = vec![
        "zyxwvutsrqponmlkzyxwvutsrqponmlk".to_string(),
        "zyxwkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
        "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
      ];
      assert_eq!(shortest_unique_prefixes(&ids), vec![5, 5, 1]);
    }

    #[test]
    fn it_returns_one_for_single_id() {
      let ids = vec!["zyxwvutsrqponmlkzyxwvutsrqponmlk".to_string()];
      assert_eq!(shortest_unique_prefixes(&ids), vec![1]);
    }
  }
}
