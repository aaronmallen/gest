use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use yansi::Paint;

use super::theme::Theme;

/// Render a markdown string to styled terminal output with word wrapping.
pub fn render(text: &str, theme: &Theme, width: usize) -> String {
  let parser = Parser::new(text);
  let mut output = String::new();
  let mut in_code_block = false;
  let mut in_heading = false;
  let mut in_blockquote = false;
  let mut in_emphasis = false;
  let mut in_strong = false;
  let mut in_link = false;
  let mut link_url = String::new();

  for event in parser {
    match event {
      Event::Start(Tag::Heading {
        ..
      }) => {
        in_heading = true;
      }
      Event::End(TagEnd::Heading(_)) => {
        in_heading = false;
        output.push('\n');
      }
      Event::Start(Tag::CodeBlock(_)) => {
        in_code_block = true;
      }
      Event::End(TagEnd::CodeBlock) => {
        in_code_block = false;
      }
      Event::Start(Tag::BlockQuote(_)) => {
        in_blockquote = true;
      }
      Event::End(TagEnd::BlockQuote(_)) => {
        in_blockquote = false;
      }
      Event::Start(Tag::Emphasis) => in_emphasis = true,
      Event::End(TagEnd::Emphasis) => in_emphasis = false,
      Event::Start(Tag::Strong) => in_strong = true,
      Event::End(TagEnd::Strong) => in_strong = false,
      Event::Start(Tag::Link {
        dest_url,
        id: _,
        ..
      }) => {
        in_link = true;
        link_url = dest_url.to_string();
      }
      Event::End(TagEnd::Link) => {
        in_link = false;
        link_url = String::new();
      }
      Event::Code(code) => {
        output.push_str(&format!("{}", code.paint(theme.markdown_code_inline)));
      }
      Event::Text(text) => {
        if in_code_block {
          for line in text.lines() {
            output.push_str(&format!(
              "  {} {}\n",
              "\u{2502}".paint(theme.markdown_code_border),
              line.paint(theme.markdown_code_block),
            ));
          }
        } else if in_heading {
          output.push_str(&format!("{}", text.paint(theme.markdown_heading)));
        } else if in_blockquote {
          for line in text.lines() {
            output.push_str(&format!(
              "  {} {}\n",
              ">".paint(theme.markdown_blockquote_border),
              line.paint(theme.markdown_blockquote),
            ));
          }
        } else if in_link {
          let styled = text.paint(theme.markdown_link);
          if yansi::is_enabled() {
            output.push_str(&format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", link_url, styled));
          } else {
            output.push_str(&format!("{styled}"));
          }
        } else if in_emphasis {
          output.push_str(&format!("{}", text.paint(theme.markdown_emphasis)));
        } else if in_strong {
          output.push_str(&format!("{}", text.paint(theme.markdown_strong)));
        } else {
          output.push_str(&word_wrap(&text, width));
        }
      }
      Event::SoftBreak | Event::HardBreak => {
        output.push('\n');
      }
      Event::Rule => {
        let line: String = std::iter::repeat_n('\u{2500}', width).collect();
        output.push_str(&format!("{}\n", line.paint(theme.markdown_rule)));
      }
      Event::Start(Tag::Paragraph) => {}
      Event::End(TagEnd::Paragraph) => {
        output.push_str("\n\n");
      }
      _ => {}
    }
  }

  while output.ends_with('\n') {
    output.pop();
  }

  output
}

fn word_wrap(text: &str, width: usize) -> String {
  if width == 0 {
    return text.to_string();
  }

  let mut result = String::new();
  let mut line_width = 0;

  for word in text.split_whitespace() {
    let word_width = word.len();
    if line_width > 0 && line_width + 1 + word_width > width {
      result.push('\n');
      line_width = 0;
    }
    if line_width > 0 {
      result.push(' ');
      line_width += 1;
    }
    result.push_str(word);
    line_width += word_width;
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  fn plain_theme() -> Theme {
    Theme::default()
  }

  fn render_plain(md: &str, width: usize) -> String {
    let _guard = yansi::disable();
    render(md, &plain_theme(), width)
  }

  mod render {
    use super::*;

    #[test]
    fn it_renders_blockquote_with_border() {
      let md = "> This is a quote.";
      let out = render_plain(md, 80);
      assert!(out.contains("> This is a quote."), "got: {out}");
    }

    #[test]
    fn it_renders_code_block_with_left_border() {
      let md = "```\nlet x = 1;\nlet y = 2;\n```";
      let out = render_plain(md, 80);
      assert!(out.contains("\u{2502} let x = 1;"), "got: {out}");
      assert!(out.contains("\u{2502} let y = 2;"), "got: {out}");
    }

    #[test]
    fn it_renders_emphasis() {
      let md = "This is *italic* text";
      let out = render_plain(md, 80);
      assert!(out.contains("italic"), "got: {out}");
    }

    #[test]
    fn it_renders_heading_text() {
      let out = render_plain("## openai streaming adapter", 80);
      assert!(out.contains("openai streaming adapter"), "got: {out}");
    }

    #[test]
    fn it_renders_horizontal_rule() {
      let out = render_plain("---", 20);
      let expected: String = std::iter::repeat_n('\u{2500}', 20).collect();
      assert!(out.contains(&expected), "got: {out}");
    }

    #[test]
    fn it_renders_inline_code() {
      let out = render_plain("Call `complete()` here", 80);
      assert!(out.contains("complete()"), "got: {out}");
    }

    #[test]
    fn it_renders_link_text() {
      let md = "[click](https://example.com)";
      let out = render_plain(md, 80);
      assert!(out.contains("click"), "got: {out}");
    }

    #[test]
    fn it_renders_strong() {
      let md = "This is **bold** text";
      let out = render_plain(md, 80);
      assert!(out.contains("bold"), "got: {out}");
    }

    #[test]
    fn it_returns_empty_for_empty_input() {
      let out = render_plain("", 80);
      assert!(out.is_empty(), "got: {out}");
    }

    #[test]
    fn it_separates_multiple_paragraphs() {
      let md = "First paragraph.\n\nSecond paragraph.";
      let out = render_plain(md, 80);
      assert!(out.contains("First paragraph."));
      assert!(out.contains("Second paragraph."));
      assert!(out.contains("\n\n"));
    }
  }

  mod word_wrap {
    use super::*;

    #[test]
    fn it_breaks_long_lines() {
      let result = word_wrap("one two three four five", 10);
      assert!(result.contains('\n'), "expected line break, got: {result}");
    }

    #[test]
    fn it_returns_original_when_width_is_zero() {
      let result = word_wrap("hello world", 0);
      assert_eq!(result, "hello world");
    }
  }
}
