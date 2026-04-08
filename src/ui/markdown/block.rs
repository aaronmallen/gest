//! Intermediate representation for parsed markdown.
//!
//! The parse pass produces a `Vec<Block>` which the render pass walks to
//! produce styled terminal output. Splitting parsing from rendering keeps each
//! step small and independently testable.

use pulldown_cmark::{BlockQuoteKind, HeadingLevel};

/// A top-level markdown block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
  BlockQuote {
    blocks: Vec<Block>,
    kind: Option<BlockQuoteKind>,
  },
  CodeBlock {
    content: String,
    lang: Option<String>,
  },
  Heading {
    inlines: Vec<Inline>,
    level: HeadingLevel,
  },
  List {
    items: Vec<Vec<Block>>,
    ordered: bool,
  },
  Paragraph(Vec<Inline>),
  Rule,
}

/// An inline element inside a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
  Code(String),
  Emphasis(Vec<Inline>),
  Link { text: Vec<Inline>, url: String },
  Strong(Vec<Inline>),
  Text(String),
}
