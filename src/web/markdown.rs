//! Markdown-to-HTML renderer for the web layer using pulldown-cmark.

use pulldown_cmark::{Options, Parser, html};

/// Convert a markdown string to sanitized HTML suitable for template rendering.
pub fn render_markdown_to_html(text: &str) -> String {
  let mut opts = Options::empty();
  opts.insert(Options::ENABLE_TABLES);
  opts.insert(Options::ENABLE_STRIKETHROUGH);
  opts.insert(Options::ENABLE_TASKLISTS);
  let parser = Parser::new_ext(text, opts);
  let mut html_output = String::new();
  html::push_html(&mut html_output, parser);
  html_output
}
