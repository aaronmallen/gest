//! Shared helpers used across multiple CLI command implementations.

/// Extract the text of the first markdown `# heading` from the input.
///
/// Returns `None` when no non-empty top-level heading is present. Lines with
/// leading or trailing whitespace are trimmed before the `# ` prefix check, so
/// indented headings are still recognized.
pub fn extract_heading(input: &str) -> Option<String> {
  for line in input.lines() {
    let trimmed = line.trim();
    if let Some(heading) = trimmed.strip_prefix("# ") {
      let heading = heading.trim();
      if !heading.is_empty() {
        return Some(heading.to_string());
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  mod extract_heading_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_extracts_heading_from_markdown() {
      let input = "# Fix the bug\n\ndetails here";

      assert_eq!(extract_heading(input), Some("Fix the bug".into()));
    }

    #[test]
    fn it_extracts_heading_after_blank_lines() {
      let input = "\n\n\n# Delayed heading\n\nbody";

      assert_eq!(extract_heading(input), Some("Delayed heading".into()));
    }

    #[test]
    fn it_returns_none_for_empty_input() {
      assert_eq!(extract_heading(""), None);
    }

    #[test]
    fn it_returns_none_when_no_heading() {
      let input = "just some text\nno heading here";

      assert_eq!(extract_heading(input), None);
    }

    #[test]
    fn it_skips_empty_headings() {
      let input = "# \n# Real heading";

      assert_eq!(extract_heading(input), Some("Real heading".into()));
    }

    #[test]
    fn it_ignores_deeper_heading_levels() {
      let input = "## not this\n### or this\n# yes this";

      assert_eq!(extract_heading(input), Some("yes this".into()));
    }
  }
}
