//! Markdown-to-styled-terminal renderer.
//!
//! Two-pass pipeline: [`parse`] converts markdown into a typed [`Block`] IR,
//! then [`render`] walks that IR to produce styled terminal output. The IR
//! split keeps each pass small and independently testable.

mod block;
mod parse;
mod render;

/// Render a markdown string to styled terminal output, fitted to `width` columns.
pub fn render(input: &str, width: usize) -> String {
  render::render(&parse::parse(input), width)
}
