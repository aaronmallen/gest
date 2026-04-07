//! Markdown-to-styled-terminal renderer using pulldown-cmark.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use yansi::Paint;

use crate::ui::style;

/// Render markdown to styled terminal output.
pub fn render(input: &str) -> String {
  let theme = style::global();
  let parser = Parser::new_ext(input, Options::all());
  let mut output = String::new();
  let mut in_code_block = false;
  let mut in_heading = false;
  let mut list_depth: usize = 0;
  let mut ordered_index: Vec<usize> = Vec::new();

  for event in parser {
    match event {
      Event::Start(Tag::Heading {
        level, ..
      }) => {
        in_heading = true;
        let prefix = match level {
          HeadingLevel::H1 => "# ",
          HeadingLevel::H2 => "## ",
          HeadingLevel::H3 => "### ",
          HeadingLevel::H4 => "#### ",
          _ => "##### ",
        };
        output.push_str(&prefix.paint(*theme.markdown_heading()).to_string());
      }
      Event::End(TagEnd::Heading(_)) => {
        in_heading = false;
        output.push('\n');
      }
      Event::Start(Tag::Paragraph) => {}
      Event::End(TagEnd::Paragraph) => {
        output.push('\n');
      }
      Event::Start(Tag::CodeBlock(_)) => {
        in_code_block = true;
      }
      Event::End(TagEnd::CodeBlock) => {
        in_code_block = false;
      }
      Event::Start(Tag::BlockQuote(_)) => {
        output.push_str(&"│ ".paint(*theme.markdown_blockquote_border()).to_string());
      }
      Event::End(TagEnd::BlockQuote(_)) => {
        output.push('\n');
      }
      Event::Start(Tag::List(Some(start))) => {
        ordered_index.push(start as usize);
        list_depth += 1;
      }
      Event::Start(Tag::List(None)) => {
        ordered_index.push(0);
        list_depth += 1;
      }
      Event::End(TagEnd::List(_)) => {
        list_depth = list_depth.saturating_sub(1);
        ordered_index.pop();
      }
      Event::Start(Tag::Item) => {
        let indent = "  ".repeat(list_depth.saturating_sub(1));
        if let Some(idx) = ordered_index.last_mut() {
          if *idx > 0 {
            output.push_str(&format!("{indent}{}. ", idx));
            *idx += 1;
          } else {
            output.push_str(&format!("{indent}• "));
          }
        }
      }
      Event::End(TagEnd::Item) => {
        output.push('\n');
      }
      Event::Start(Tag::Emphasis) => {
        output.push_str("\x1b[3m");
      }
      Event::End(TagEnd::Emphasis) => {
        output.push_str("\x1b[23m");
      }
      Event::Start(Tag::Strong) => {
        output.push_str("\x1b[1m");
      }
      Event::End(TagEnd::Strong) => {
        output.push_str("\x1b[22m");
      }
      Event::Start(Tag::Link {
        dest_url, ..
      }) => {
        output.push_str(&dest_url.paint(*theme.id_rest()).to_string());
        output.push(' ');
      }
      Event::End(TagEnd::Link) => {}
      Event::Text(text) => {
        if in_code_block {
          for line in text.lines() {
            output.push_str(&format!("  {}", line.paint(*theme.markdown_code_block())));
            output.push('\n');
          }
        } else if in_heading {
          output.push_str(&text.paint(*theme.markdown_heading()).to_string());
        } else {
          output.push_str(&text);
        }
      }
      Event::Code(code) => {
        output.push_str(&format!("`{}`", code.paint(*theme.markdown_code_inline())));
      }
      Event::SoftBreak | Event::HardBreak => {
        output.push('\n');
      }
      _ => {}
    }
  }

  output
}

#[cfg(test)]
mod tests {
  use super::*;

  mod render_fn {
    use super::*;

    #[test]
    fn it_renders_plain_text() {
      let result = render("Hello world");

      assert!(result.contains("Hello world"));
    }

    #[test]
    fn it_renders_headings() {
      let result = render("# Title");

      assert!(result.contains("Title"));
    }

    #[test]
    fn it_renders_code_blocks() {
      let result = render("```\nlet x = 1;\n```");

      assert!(result.contains("let x = 1;"));
    }

    #[test]
    fn it_renders_inline_code() {
      let result = render("Use `foo` here");

      assert!(result.contains("`"));
      assert!(result.contains("foo"));
    }

    #[test]
    fn it_renders_unordered_lists() {
      let result = render("- one\n- two");

      assert!(result.contains("•"));
      assert!(result.contains("one"));
      assert!(result.contains("two"));
    }

    #[test]
    fn it_renders_ordered_lists() {
      let result = render("1. first\n2. second");

      assert!(result.contains("1."));
      assert!(result.contains("first"));
    }
  }
}
