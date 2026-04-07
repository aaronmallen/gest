use std::fmt::{self, Display, Formatter};

use super::super::atoms::{Badge, Column, Id, Tag, Title};
use crate::ui::{components::molecules::Row, style};

/// Max display width for artifact titles in list rows.
const TITLE_PAD: usize = 35;

/// A single row in an artifact list, showing id, title, tags, and optional archived badge.
pub struct Component {
  id: String,
  id_prefix_len: usize,
  is_archived: bool,
  tags: Vec<String>,
  title: String,
}

impl Component {
  pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      id_prefix_len: 2,
      is_archived: false,
      tags: Vec::new(),
      title: title.into(),
    }
  }

  /// Marks this row as archived, applying dimmed styles and appending an `[archived]` badge.
  pub fn archived(mut self) -> Self {
    self.is_archived = true;
    self
  }

  /// Sets the number of highlighted prefix characters in the ID.
  pub fn id_prefix_len(mut self, len: usize) -> Self {
    self.id_prefix_len = len;
    self
  }

  pub fn tags(mut self, tags: Vec<String>) -> Self {
    self.tags = tags;
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = style::global();
    let mut row = Row::new().spacing(2);

    row = row.col(Column::natural(Id::new(&self.id).prefix_len(self.id_prefix_len)));

    let title_style = if self.is_archived {
      *theme.artifact_list_title_archived()
    } else {
      *theme.artifact_list_title()
    };
    row = row.col(Column::natural(
      Title::new(&self.title, title_style)
        .max_width(TITLE_PAD)
        .pad_to(TITLE_PAD),
    ));

    let tag_style = if self.is_archived {
      *theme.artifact_list_tag_archived()
    } else {
      *theme.tag()
    };
    if !self.tags.is_empty() {
      row = row.col(Column::natural(Tag::new(self.tags.clone(), tag_style)));
    }

    if self.is_archived {
      row = row.col(Column::natural(Badge::new(
        "[archived]",
        *theme.artifact_list_archived_badge(),
      )));
    }

    write!(f, "{row}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_renders_archived_row_with_badge() {
    yansi::disable();
    let tags = vec!["spec".to_string(), "backend".to_string()];
    let row = Component::new("pkzqwrnd", "auth-spec").tags(tags).archived();
    let rendered = row.to_string();

    assert!(rendered.contains("pkzqwrnd"));
    assert!(rendered.contains("auth-spec"));
    assert!(rendered.contains("#spec"));
    assert!(rendered.contains("#backend"));
    assert!(rendered.contains("[archived]"));
  }

  #[test]
  fn it_renders_normal_row_with_id_title_and_tags() {
    yansi::disable();
    let tags = vec!["schema".to_string()];
    let row = Component::new("fsahdqlt", "probe-schema-v2").tags(tags);
    let rendered = row.to_string();

    assert!(rendered.contains("fsahdqlt"));
    assert!(rendered.contains("probe-schema-v2"));
    assert!(rendered.contains("#schema"));
    assert!(!rendered.contains("[archived]"));
  }

  #[test]
  fn it_renders_row_with_multiple_tags() {
    yansi::disable();
    let tags = vec!["spec".to_string(), "backend".to_string(), "v2".to_string()];
    let row = Component::new("abcdefgh", "my-artifact").tags(tags);
    let rendered = row.to_string();

    assert!(rendered.contains("#spec"));
    assert!(rendered.contains("#backend"));
    assert!(rendered.contains("#v2"));
    assert!(rendered.contains("#spec  #backend  #v2"));
  }

  #[test]
  fn it_renders_row_with_no_tags() {
    yansi::disable();
    let row = Component::new("abcdefgh", "bare-artifact");
    let rendered = row.to_string();

    assert!(rendered.contains("abcdefgh"));
    assert!(rendered.contains("bare-artifact"));
    assert!(!rendered.contains('#'));
  }
}
