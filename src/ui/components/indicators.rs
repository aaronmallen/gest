use std::fmt;

use yansi::Paint;

use crate::{
  model::link::{Link, RelationshipType},
  ui::theme::Theme,
};

/// Atomic component for rendering task block/dependency indicators.
///
/// Renders:
/// - `!!` when the task is blocked by another task
/// - `⚠ N` when the task blocks N other tasks
/// - Both indicators separated by a space when applicable
pub struct Indicators<'a> {
  links: &'a [Link],
  theme: &'a Theme,
}

impl<'a> Indicators<'a> {
  pub fn new(links: &'a [Link], theme: &'a Theme) -> Self {
    Self {
      links,
      theme,
    }
  }
}

impl fmt::Display for Indicators<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let is_blocked = self.links.iter().any(|l| l.rel == RelationshipType::BlockedBy);
    let blocks_count = self.links.iter().filter(|l| l.rel == RelationshipType::Blocks).count();

    let mut parts = Vec::new();
    if is_blocked {
      parts.push("!!".paint(self.theme.indicator_blocked).to_string());
    }
    if blocks_count > 0 {
      parts.push(
        format!("\u{26a0} {}", blocks_count)
          .paint(self.theme.indicator_blocking)
          .to_string(),
      );
    }

    write!(f, "{}", parts.join(" "))
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  fn make_link(rel: RelationshipType) -> Link {
    Link {
      ref_: "tasks/kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk".to_string(),
      rel,
    }
  }

  #[test]
  fn it_renders_empty_when_no_indicators() {
    let links: Vec<Link> = vec![];
    let theme = Theme::default();
    let component = Indicators::new(&links, &theme);
    let rendered = component.to_string();
    assert_eq!(rendered, "");
  }

  #[test]
  fn it_renders_blocked_indicator() {
    let links = vec![make_link(RelationshipType::BlockedBy)];
    let theme = Theme::default();
    let component = Indicators::new(&links, &theme);
    let rendered = component.to_string();
    assert!(rendered.contains("!!"), "Should contain blocked indicator");
    assert!(!rendered.contains('\u{26a0}'), "Should not contain blocking indicator");
  }

  #[test]
  fn it_renders_blocking_indicator_with_count() {
    let links = vec![
      make_link(RelationshipType::Blocks),
      Link {
        ref_: "tasks/lllllllllllllllllllllllllllllllu".to_string(),
        rel: RelationshipType::Blocks,
      },
    ];
    let theme = Theme::default();
    let component = Indicators::new(&links, &theme);
    let rendered = component.to_string();
    assert!(!rendered.contains("!!"), "Should not contain blocked indicator");
    assert!(rendered.contains('\u{26a0}'), "Should contain warning symbol");
    assert!(rendered.contains('2'), "Should show count of 2");
  }

  #[test]
  fn it_renders_both_indicators() {
    let links = vec![
      make_link(RelationshipType::BlockedBy),
      make_link(RelationshipType::Blocks),
    ];
    let theme = Theme::default();
    let component = Indicators::new(&links, &theme);
    let rendered = component.to_string();
    assert!(rendered.contains("!!"), "Should contain blocked indicator");
    assert!(rendered.contains('\u{26a0}'), "Should contain blocking indicator");
    assert!(rendered.contains('1'), "Should show count of 1");
  }

  #[test]
  fn it_ignores_non_block_relationships() {
    let links = vec![
      make_link(RelationshipType::ChildOf),
      make_link(RelationshipType::RelatesTo),
    ];
    let theme = Theme::default();
    let component = Indicators::new(&links, &theme);
    let rendered = component.to_string();
    assert_eq!(rendered, "");
  }
}
