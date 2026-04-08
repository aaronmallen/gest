//! Render pass: convert the [`Block`] IR into styled terminal output.

use pulldown_cmark::BlockQuoteKind;
use unicode_width::UnicodeWidthStr;
use yansi::{Paint, Style};

use super::block::{Block, Inline};
use crate::ui::style;

/// Render a list of blocks to a styled string fitted to `width` columns.
pub fn render(blocks: &[Block], width: usize) -> String {
  let mut output = String::new();
  for (i, block) in blocks.iter().enumerate() {
    if i > 0 {
      output.push('\n');
    }
    output.push_str(&render_block(block, width));
  }
  output
}

/// Render a single block.
fn render_block(block: &Block, width: usize) -> String {
  match block {
    Block::BlockQuote {
      blocks,
      kind,
    } => render_block_quote(blocks, *kind, width),
    Block::CodeBlock {
      content, ..
    } => render_code_block(content),
    Block::Heading {
      inlines, ..
    } => render_heading(inlines),
    Block::List {
      items,
      ordered,
    } => render_list(items, *ordered, width),
    Block::Paragraph(inlines) => render_paragraph(inlines, width),
    Block::Rule => render_rule(width),
  }
}

/// Render a block quote with `| ` borders on every line, dispatching colour by alert kind.
fn render_block_quote(blocks: &[Block], kind: Option<BlockQuoteKind>, width: usize) -> String {
  let theme = style::global();
  let border_style = match kind {
    None => *theme.markdown_blockquote_border(),
    Some(BlockQuoteKind::Note) => *theme.markdown_alert_note_border(),
    Some(BlockQuoteKind::Tip) => *theme.markdown_alert_tip_border(),
    Some(BlockQuoteKind::Important) => *theme.markdown_alert_important_border(),
    Some(BlockQuoteKind::Warning) => *theme.markdown_alert_warning_border(),
    Some(BlockQuoteKind::Caution) => *theme.markdown_alert_caution_border(),
  };

  let inner_width = width.saturating_sub(2);
  let inner = render(blocks, inner_width);
  let border = "| ".paint(border_style).to_string();

  let mut out = String::new();
  for line in inner.lines() {
    out.push_str(&border);
    out.push_str(line);
    out.push('\n');
  }
  out
}

/// Render a fenced code block with a left border. Lines are emitted verbatim — no wrapping.
fn render_code_block(content: &str) -> String {
  let theme = style::global();
  let border = "| ".paint(*theme.markdown_code_border()).to_string();
  let body_style = *theme.markdown_code_block();

  let mut out = String::new();
  for line in content.lines() {
    out.push_str(&border);
    out.push_str(&line.paint(body_style).to_string());
    out.push('\n');
  }
  out
}

/// Render a heading: styled text followed by a blank line.
fn render_heading(inlines: &[Inline]) -> String {
  let theme = style::global();
  let text = render_inlines(inlines);
  format!("{}\n\n", text.paint(*theme.markdown_heading()))
}

/// Render a single inline element.
fn render_inline(inline: &Inline) -> String {
  let theme = style::global();
  match inline {
    Inline::Code(code) => code.paint(*theme.markdown_code_inline()).to_string(),
    Inline::Emphasis(children) => {
      let inner = render_inlines(children);
      inner.paint(*theme.markdown_emphasis()).to_string()
    }
    Inline::Link {
      text,
      url,
    } => render_link(text, url, *theme.markdown_link()),
    Inline::Strong(children) => {
      let inner = render_inlines(children);
      inner.paint(*theme.markdown_strong()).to_string()
    }
    Inline::Text(text) => text.clone(),
  }
}

/// Render a sequence of inlines into a single styled string.
fn render_inlines(inlines: &[Inline]) -> String {
  let mut out = String::new();
  for inline in inlines {
    out.push_str(&render_inline(inline));
  }
  out
}

/// Render a link as an OSC 8 hyperlink escape sequence wrapping the styled text.
fn render_link(text: &[Inline], url: &str, link_style: Style) -> String {
  let label = render_inlines(text);
  let styled = label.paint(link_style);
  if yansi::is_enabled() {
    format!("\x1b]8;;{url}\x07{styled}\x1b]8;;\x07")
  } else {
    styled.to_string()
  }
}

/// Render a list, recursively rendering each item's blocks with a marker prefix.
fn render_list(items: &[Vec<Block>], ordered: bool, width: usize) -> String {
  let mut out = String::new();
  for (i, item) in items.iter().enumerate() {
    let marker = if ordered {
      format!("{}. ", i + 1)
    } else {
      "• ".to_string()
    };
    let indent = " ".repeat(marker.width());
    let inner_width = width.saturating_sub(marker.width());
    let body = render(item, inner_width);

    let mut first = true;
    for line in body.lines() {
      if first {
        out.push_str(&marker);
        first = false;
      } else {
        out.push_str(&indent);
      }
      out.push_str(line);
      out.push('\n');
    }
  }
  out
}

/// Render a paragraph: word-wrap inlines to `width` and emit a trailing newline.
fn render_paragraph(inlines: &[Inline], width: usize) -> String {
  let text = render_inlines(inlines);
  let mut out = word_wrap(&text, width);
  out.push('\n');
  out
}

/// Render a horizontal rule as a styled `─` line spanning `width`.
fn render_rule(width: usize) -> String {
  let theme = style::global();
  let line: String = std::iter::repeat_n('─', width).collect();
  format!("{}\n", line.paint(*theme.markdown_rule()))
}

/// Word-wrap a string to fit within `width` columns. Width 0 disables wrapping.
///
/// This is intentionally width-aware (uses display width, not byte length) but
/// does not understand ANSI escape sequences embedded in the input — styled
/// inlines may push lines slightly past `width`. Wrapping is best-effort for
/// human readability.
fn word_wrap(text: &str, width: usize) -> String {
  if width == 0 {
    return text.to_string();
  }

  let mut result = String::new();
  for (i, paragraph) in text.split('\n').enumerate() {
    if i > 0 {
      result.push('\n');
    }
    let mut line_width = 0;
    let mut first_word = true;
    for word in paragraph.split_whitespace() {
      let word_width = word.width();
      if !first_word && line_width + 1 + word_width > width {
        result.push('\n');
        line_width = 0;
        first_word = true;
      }
      if !first_word {
        result.push(' ');
        line_width += 1;
      }
      result.push_str(word);
      line_width += word_width;
      first_word = false;
    }
  }
  result
}

#[cfg(test)]
mod tests {
  use std::sync::{Mutex, OnceLock};

  use super::*;
  use crate::ui::markdown::parse::parse;

  fn render_md(input: &str, width: usize) -> String {
    let _guard = yansi_lock();
    strip_ansi(&render(&parse(input), width))
  }

  fn render_md_ansi(input: &str, width: usize) -> String {
    let _guard = yansi_lock();
    yansi::enable();
    render(&parse(input), width)
  }

  /// Strip ANSI escape sequences (CSI and OSC) from a styled string for
  /// content-level assertions that don't care about the styling itself.
  fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
      if c == '\x1b' {
        match chars.next() {
          Some('[') => {
            for next in chars.by_ref() {
              if next.is_ascii_alphabetic() {
                break;
              }
            }
          }
          Some(']') => {
            for next in chars.by_ref() {
              if next == '\x07' {
                break;
              }
            }
          }
          _ => {}
        }
      } else {
        out.push(c);
      }
    }
    out
  }

  /// Serializes tests that mutate yansi's global enable flag to keep parallel
  /// tests from clobbering each other.
  fn yansi_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK
      .get_or_init(|| Mutex::new(()))
      .lock()
      .unwrap_or_else(|e| e.into_inner())
  }

  mod render_fn {
    use super::*;

    #[test]
    fn it_does_not_wrap_code_blocks() {
      let long = "a".repeat(100);
      let md = format!("```\n{long}\n```");
      let out = render_md(&md, 20);

      assert!(out.contains(&long), "code block should not be wrapped, got: {out:?}");
    }

    #[test]
    fn it_renders_alert_caution_with_distinct_color() {
      let out = render_md_ansi("> [!CAUTION]\n> danger", 80);
      let theme = style::global();
      let expected = "| ".paint(*theme.markdown_alert_caution_border()).to_string();

      assert!(out.contains(&expected));
    }

    #[test]
    fn it_renders_alert_important_with_distinct_color() {
      let out = render_md_ansi("> [!IMPORTANT]\n> read this", 80);
      let theme = style::global();
      let expected = "| ".paint(*theme.markdown_alert_important_border()).to_string();

      assert!(out.contains(&expected));
    }

    #[test]
    fn it_renders_alert_note_with_distinct_color() {
      let out = render_md_ansi("> [!NOTE]\n> heads up", 80);
      let theme = style::global();
      let expected = "| ".paint(*theme.markdown_alert_note_border()).to_string();

      assert!(
        out.contains(&expected),
        "note alert border should match theme, got: {out:?}"
      );
    }

    #[test]
    fn it_renders_alert_tip_with_distinct_color() {
      let out = render_md_ansi("> [!TIP]\n> nice", 80);
      let theme = style::global();
      let expected = "| ".paint(*theme.markdown_alert_tip_border()).to_string();

      assert!(out.contains(&expected));
    }

    #[test]
    fn it_renders_alert_warning_with_distinct_color() {
      let out = render_md_ansi("> [!WARNING]\n> careful", 80);
      let theme = style::global();
      let expected = "| ".paint(*theme.markdown_alert_warning_border()).to_string();

      assert!(out.contains(&expected));
    }

    #[test]
    fn it_renders_block_quote_border_on_every_line() {
      let out = render_md("> first paragraph\n>\n> second paragraph\n>\n> third paragraph", 80);
      let border_lines = out.lines().filter(|l| l.starts_with("| ")).count();

      assert!(
        border_lines >= 3,
        "every paragraph line should have border, got: {out:?}"
      );
    }

    #[test]
    fn it_renders_code_block_with_left_border() {
      let out = render_md("```\nlet x = 1;\nlet y = 2;\n```", 80);

      assert!(out.contains("| let x = 1;"), "got: {out:?}");
      assert!(out.contains("| let y = 2;"), "got: {out:?}");
    }

    #[test]
    fn it_renders_heading_with_blank_line_after() {
      let out = render_md("# Title\n\nbody", 80);

      assert!(out.contains("Title"));
      assert!(
        out.contains("Title\n\n"),
        "heading should be followed by blank line, got: {out:?}"
      );
    }

    #[test]
    fn it_renders_horizontal_rule_full_width() {
      let out = render_md("---", 20);
      let expected: String = std::iter::repeat_n('─', 20).collect();

      assert!(out.contains(&expected), "rule should span full width, got: {out:?}");
    }

    #[test]
    fn it_renders_inline_code() {
      let out = render_md("Use `foo` here", 80);

      assert!(out.contains("foo"));
    }

    #[test]
    fn it_renders_link_with_osc8_escape() {
      let out = render_md_ansi("[click](https://example.com)", 80);

      assert!(
        out.contains("\x1b]8;;https://example.com\x07"),
        "link should include OSC 8 prefix, got: {out:?}"
      );
      assert!(out.contains("click"), "link text should be present, got: {out:?}");
      assert!(
        out.contains("\x1b]8;;\x07"),
        "link should include OSC 8 terminator, got: {out:?}"
      );
    }

    #[test]
    fn it_renders_ordered_list() {
      let out = render_md("1. first\n2. second", 80);

      assert!(out.contains("1. first"));
      assert!(out.contains("2. second"));
    }

    #[test]
    fn it_renders_plain_paragraph() {
      let out = render_md("Hello world", 80);

      assert!(out.contains("Hello world"));
    }

    #[test]
    fn it_renders_unordered_list() {
      let out = render_md("- one\n- two", 80);

      assert!(out.contains("• one"), "got: {out:?}");
      assert!(out.contains("• two"), "got: {out:?}");
    }

    #[test]
    fn it_returns_empty_for_empty_input() {
      let out = render_md("", 80);

      assert!(out.is_empty());
    }

    #[test]
    fn it_underlines_headings_in_ansi_output() {
      let out = render_md_ansi("# Title", 80);

      assert!(
        out.contains("\x1b[4m") || out.contains(";4;") || out.contains(";4m") || out.contains("\x1b[4;"),
        "heading should include underline ANSI, got: {out:?}"
      );
    }

    #[test]
    fn it_word_wraps_paragraphs() {
      let out = render_md("one two three four five six seven", 10);

      assert!(
        out.lines().count() >= 2,
        "paragraph should wrap into multiple lines, got: {out:?}"
      );
    }

    #[test]
    fn it_wraps_block_quote_lines_with_border() {
      let out = render_md("> one two three four five six seven eight nine ten", 12);
      let border_lines = out.lines().filter(|l| l.starts_with("| ")).count();

      assert!(
        border_lines >= 2,
        "wrapped block quote should keep border on every line, got: {out:?}"
      );
    }
  }

  mod word_wrap_fn {
    use super::*;

    #[test]
    fn it_breaks_long_lines() {
      let result = word_wrap("one two three four five", 10);

      assert!(result.contains('\n'));
    }

    #[test]
    fn it_returns_input_when_width_is_zero() {
      let result = word_wrap("hello world", 0);

      assert_eq!(result, "hello world");
    }
  }
}
