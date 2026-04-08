//! Intermediate representation for parsed markdown.
//!
//! The parse pass produces a `Vec<Block>` which the render pass walks to
//! produce styled terminal output. Splitting parsing from rendering keeps each
//! step small and independently testable.

use pulldown_cmark::{BlockQuoteKind, HeadingLevel};

/// A top-level markdown block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Block {
  /// A blockquote, optionally tagged as a GitHub-flavored alert.
  BlockQuote {
    /// Nested blocks contained inside the quote.
    blocks: Vec<Block>,
    /// Alert kind (note, tip, warning, etc.) when the quote is a GFM alert.
    kind: Option<BlockQuoteKind>,
  },
  /// A fenced or indented code block rendered verbatim.
  CodeBlock {
    /// Raw code content without the fence markers.
    content: String,
    /// Optional fenced language tag.
    lang: Option<String>,
  },
  /// A heading with its level and inline contents.
  Heading {
    /// Inline contents of the heading.
    inlines: Vec<Inline>,
    /// Heading level (h1 through h6).
    level: HeadingLevel,
  },
  /// An ordered or unordered list.
  List {
    /// Each item is itself a sequence of blocks.
    items: Vec<Vec<Block>>,
    /// Whether the list is numbered.
    ordered: bool,
  },
  /// A paragraph containing inline elements.
  Paragraph(Vec<Inline>),
  /// A horizontal rule (thematic break).
  Rule,
}

/// An inline element inside a block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Inline {
  /// Inline code span.
  Code(String),
  /// Italicized children.
  Emphasis(Vec<Inline>),
  /// A hyperlink with styled link text.
  Link {
    /// Inline contents rendered as the visible link text.
    text: Vec<Inline>,
    /// Destination URL for the link.
    url: String,
  },
  /// Bold children.
  Strong(Vec<Inline>),
  /// Plain text with no inline styling.
  Text(String),
}
