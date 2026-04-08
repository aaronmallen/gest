//! Parse pass: convert pulldown-cmark events into the [`Block`] IR.

use pulldown_cmark::{BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::block::{Block, Inline};

/// Internal stack frame tracking partially-parsed structures.
enum Frame {
  BlockQuote {
    blocks: Vec<Block>,
    kind: Option<BlockQuoteKind>,
  },
  CodeBlock {
    content: String,
    lang: Option<String>,
  },
  Emphasis(Vec<Inline>),
  Heading {
    inlines: Vec<Inline>,
    level: HeadingLevel,
  },
  Link {
    text: Vec<Inline>,
    url: String,
  },
  List {
    current: Option<Vec<Block>>,
    items: Vec<Vec<Block>>,
    ordered: bool,
  },
  Paragraph(Vec<Inline>),
  Root(Vec<Block>),
  Strong(Vec<Inline>),
}

/// Parse a markdown string into a flat list of top-level blocks.
pub fn parse(input: &str) -> Vec<Block> {
  let parser = Parser::new_ext(input, Options::all());
  let mut stack: Vec<Frame> = vec![Frame::Root(Vec::new())];

  for event in parser {
    match event {
      Event::Start(Tag::Heading {
        level, ..
      }) => stack.push(Frame::Heading {
        inlines: Vec::new(),
        level,
      }),
      Event::End(TagEnd::Heading(_)) => {
        if let Some(Frame::Heading {
          inlines,
          level,
        }) = stack.pop()
        {
          push_block(
            &mut stack,
            Block::Heading {
              inlines,
              level,
            },
          );
        }
      }

      Event::Start(Tag::Paragraph) => stack.push(Frame::Paragraph(Vec::new())),
      Event::End(TagEnd::Paragraph) => {
        if let Some(Frame::Paragraph(inlines)) = stack.pop() {
          push_block(&mut stack, Block::Paragraph(inlines));
        }
      }

      Event::Start(Tag::BlockQuote(kind)) => stack.push(Frame::BlockQuote {
        blocks: Vec::new(),
        kind,
      }),
      Event::End(TagEnd::BlockQuote(_)) => {
        if let Some(Frame::BlockQuote {
          blocks,
          kind,
        }) = stack.pop()
        {
          push_block(
            &mut stack,
            Block::BlockQuote {
              blocks,
              kind,
            },
          );
        }
      }

      Event::Start(Tag::CodeBlock(kind)) => {
        let lang = match kind {
          CodeBlockKind::Fenced(s) if !s.is_empty() => Some(s.to_string()),
          _ => None,
        };
        stack.push(Frame::CodeBlock {
          content: String::new(),
          lang,
        });
      }
      Event::End(TagEnd::CodeBlock) => {
        if let Some(Frame::CodeBlock {
          mut content,
          lang,
        }) = stack.pop()
        {
          if content.ends_with('\n') {
            content.pop();
          }
          push_block(
            &mut stack,
            Block::CodeBlock {
              content,
              lang,
            },
          );
        }
      }

      Event::Start(Tag::List(start)) => stack.push(Frame::List {
        current: None,
        items: Vec::new(),
        ordered: start.is_some(),
      }),
      Event::End(TagEnd::List(_)) => {
        if let Some(Frame::List {
          current,
          mut items,
          ordered,
        }) = stack.pop()
        {
          if let Some(item) = current {
            items.push(item);
          }
          push_block(
            &mut stack,
            Block::List {
              items,
              ordered,
            },
          );
        }
      }
      Event::Start(Tag::Item) => {
        if let Some(Frame::List {
          current,
          items,
          ..
        }) = stack.last_mut()
        {
          if let Some(prev) = current.take() {
            items.push(prev);
          }
          *current = Some(Vec::new());
        }
      }
      Event::End(TagEnd::Item) => {}

      Event::Start(Tag::Emphasis) => stack.push(Frame::Emphasis(Vec::new())),
      Event::End(TagEnd::Emphasis) => {
        if let Some(Frame::Emphasis(inlines)) = stack.pop() {
          push_inline(&mut stack, Inline::Emphasis(inlines));
        }
      }
      Event::Start(Tag::Strong) => stack.push(Frame::Strong(Vec::new())),
      Event::End(TagEnd::Strong) => {
        if let Some(Frame::Strong(inlines)) = stack.pop() {
          push_inline(&mut stack, Inline::Strong(inlines));
        }
      }
      Event::Start(Tag::Link {
        dest_url, ..
      }) => stack.push(Frame::Link {
        text: Vec::new(),
        url: dest_url.to_string(),
      }),
      Event::End(TagEnd::Link) => {
        if let Some(Frame::Link {
          text,
          url,
        }) = stack.pop()
        {
          push_inline(
            &mut stack,
            Inline::Link {
              text,
              url,
            },
          );
        }
      }

      Event::Text(text) => {
        if let Some(Frame::CodeBlock {
          content, ..
        }) = stack.last_mut()
        {
          content.push_str(&text);
        } else {
          push_inline(&mut stack, Inline::Text(text.into_string()));
        }
      }
      Event::Code(code) => push_inline(&mut stack, Inline::Code(code.into_string())),
      Event::SoftBreak => push_inline(&mut stack, Inline::Text(" ".to_string())),
      Event::HardBreak => push_inline(&mut stack, Inline::Text("\n".to_string())),
      Event::Rule => push_block(&mut stack, Block::Rule),

      _ => {}
    }
  }

  if let Some(Frame::Root(blocks)) = stack.pop() {
    blocks
  } else {
    Vec::new()
  }
}

/// Push a finished block into the nearest container that holds blocks.
fn push_block(stack: &mut [Frame], block: Block) {
  for frame in stack.iter_mut().rev() {
    match frame {
      Frame::Root(blocks)
      | Frame::BlockQuote {
        blocks, ..
      } => {
        blocks.push(block);
        return;
      }
      Frame::List {
        current: Some(item), ..
      } => {
        item.push(block);
        return;
      }
      _ => {}
    }
  }
}

/// Push a finished inline into the nearest container that holds inlines.
///
/// Tight list items emit inline events without an enclosing paragraph; in that
/// case we lazily create a [`Block::Paragraph`] inside the current list item so
/// that text content is not lost.
fn push_inline(stack: &mut [Frame], inline: Inline) {
  for frame in stack.iter_mut().rev() {
    match frame {
      Frame::Heading {
        inlines, ..
      }
      | Frame::Paragraph(inlines)
      | Frame::Emphasis(inlines)
      | Frame::Strong(inlines) => {
        inlines.push(inline);
        return;
      }
      Frame::Link {
        text, ..
      } => {
        text.push(inline);
        return;
      }
      Frame::List {
        current: Some(item), ..
      } => {
        if let Some(Block::Paragraph(inlines)) = item.last_mut() {
          inlines.push(inline);
        } else {
          item.push(Block::Paragraph(vec![inline]));
        }
        return;
      }
      Frame::Root(blocks)
      | Frame::BlockQuote {
        blocks, ..
      } => {
        if let Some(Block::Paragraph(inlines)) = blocks.last_mut() {
          inlines.push(inline);
        } else {
          blocks.push(Block::Paragraph(vec![inline]));
        }
        return;
      }
      _ => {}
    }
  }
}
