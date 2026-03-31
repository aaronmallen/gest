use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{atoms::separator::Separator, theme::Theme};

/// Fixed padding width for entity type labels.
const TYPE_LABEL_PAD: usize = 10;

/// Renders an expanded search result with a dashed separator header, row content, and optional snippet.
pub struct SearchResultExpanded<'a> {
  entity_type: &'a str,
  id: &'a str,
  row_content: String,
  snippet: Option<&'a str>,
  theme: &'a Theme,
}

impl<'a> SearchResultExpanded<'a> {
  pub fn new(entity_type: &'a str, id: &'a str, row_content: impl Display, theme: &'a Theme) -> Self {
    Self {
      entity_type,
      id,
      row_content: row_content.to_string(),
      snippet: None,
      theme,
    }
  }

  /// Sets optional body text to display as an indented snippet below the row.
  pub fn snippet(mut self, text: Option<&'a str>) -> Self {
    self.snippet = text;
    self
  }
}

impl Display for SearchResultExpanded<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let label = format!("{} {}", self.entity_type, self.id);
    let sep = Separator::dashed(label, self.theme.search_expand_separator);
    writeln!(f, "{sep}")?;

    write!(f, "{}", self.row_content)?;

    if let Some(text) = self.snippet {
      writeln!(f)?;
      writeln!(f)?;
      for line in text.lines() {
        writeln!(f, "  {}", line.paint(self.theme.muted))?;
      }
    }

    Ok(())
  }
}

/// Renders a compact search result row with a padded entity type label and inline content.
pub struct SearchResultRow<'a> {
  entity_type: &'a str,
  row_content: String,
  theme: &'a Theme,
}

impl<'a> SearchResultRow<'a> {
  pub fn new(entity_type: &'a str, row_content: impl Display, theme: &'a Theme) -> Self {
    Self {
      entity_type,
      row_content: row_content.to_string(),
      theme,
    }
  }
}

impl Display for SearchResultRow<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let padding = TYPE_LABEL_PAD.saturating_sub(self.entity_type.len());
    let label = self.entity_type.paint(self.theme.search_type_label);
    write!(f, "{label}{}{}", " ".repeat(padding), self.row_content)
  }
}

/// Renders a search results count line, with a hint when no results are found.
pub struct SearchSummary<'a> {
  count: usize,
  query: &'a str,
  theme: &'a Theme,
}

impl<'a> SearchSummary<'a> {
  pub fn new(count: usize, query: &'a str, theme: &'a Theme) -> Self {
    Self {
      count,
      query,
      theme,
    }
  }
}

impl Display for SearchSummary<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let n = self.count;
    let label = if n == 1 { "result" } else { "results" };
    let query_str = format!("\"{}\"", self.query);
    let query = query_str.paint(self.theme.search_query);

    let summary = format!("{n} {label} for {query}");
    write!(f, "{}", summary.paint(self.theme.search_summary))?;

    if self.count == 0 {
      write!(
        f,
        "  {}",
        "(try broadening your query)".paint(self.theme.search_no_results_hint),
      )?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(item: &impl Display) -> String {
    yansi::disable();
    let out = item.to_string();
    yansi::enable();
    out
  }

  mod search_result_expanded {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_contains_separator_and_content() {
        let theme = theme();
        let expanded = SearchResultExpanded::new("task", "hpvrlbme", "row content here", &theme);
        let out = render(&expanded);

        assert!(out.contains("task hpvrlbme"), "separator should have type and id");
        assert!(out.contains('\u{254C}'), "should use dashed character");
        assert!(out.contains("row content here"), "should contain row content");
      }

      #[test]
      fn it_indents_multiline_snippet() {
        let theme = theme();
        let snippet = "Line one of the description.\nLine two continues here.";
        let expanded = SearchResultExpanded::new("task", "abcdefgh", "row", &theme).snippet(Some(snippet));
        let out = render(&expanded);

        assert!(out.contains("  Line one"), "first snippet line indented");
        assert!(out.contains("  Line two"), "second snippet line indented");
      }

      #[test]
      fn it_renders_with_snippet() {
        let theme = theme();
        let expanded = SearchResultExpanded::new("task", "hpvrlbme", "row line", &theme)
          .snippet(Some("Defines the canonical probe schema."));
        let out = render(&expanded);

        assert!(
          out.contains("Defines the canonical probe schema."),
          "should contain snippet text"
        );
        assert!(out.contains("  Defines"), "snippet should be indented with 2 spaces");
      }

      #[test]
      fn it_renders_without_snippet() {
        let theme = theme();
        let expanded = SearchResultExpanded::new("artifact", "fsahdqlt", "artifact row", &theme);
        let out = render(&expanded);

        assert!(out.contains("artifact fsahdqlt"), "separator label correct");
        assert!(out.contains("artifact row"), "row content present");
      }
    }
  }

  mod search_result_row {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_contains_type_label_and_content() {
        let theme = theme();
        let row = SearchResultRow::new("task", "nfkbqmrx  openai streaming adapter", &theme);
        let out = render(&row);

        assert!(out.contains("task"), "should contain type label");
        assert!(
          out.contains("nfkbqmrx  openai streaming adapter"),
          "should contain row content"
        );
      }

      #[test]
      fn it_pads_type_label() {
        let theme = theme();
        let row = SearchResultRow::new("task", "content-here", &theme);
        let out = render(&row);

        assert!(out.starts_with("task      "), "should pad type label");
      }

      #[test]
      fn it_renders_artifact_type() {
        let theme = theme();
        let row = SearchResultRow::new("artifact", "fsahdqlt  probe-schema-v2", &theme);
        let out = render(&row);

        assert!(out.contains("artifact"), "should contain artifact label");
        assert!(out.contains("fsahdqlt"), "should contain artifact id");
      }
    }
  }

  mod search_summary {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_plural_results() {
        let theme = theme();
        let summary = SearchSummary::new(3, "schema", &theme);
        let out = render(&summary);

        assert!(out.contains("3 results for"), "should show count");
        assert!(out.contains("\"schema\""), "should show quoted query");
        assert!(!out.contains("broadening"), "no hint for non-zero count");
      }

      #[test]
      fn it_renders_singular_result() {
        let theme = theme();
        let summary = SearchSummary::new(1, "auth", &theme);
        let out = render(&summary);

        assert!(out.contains("1 result for"), "should use singular");
        assert!(!out.contains("results"), "should not use plural");
      }

      #[test]
      fn it_shows_hint_for_zero_results() {
        let theme = theme();
        let summary = SearchSummary::new(0, "auth", &theme);
        let out = render(&summary);

        assert!(out.contains("0 results for"), "should show zero count");
        assert!(out.contains("try broadening your query"), "should show hint");
      }
    }
  }
}
