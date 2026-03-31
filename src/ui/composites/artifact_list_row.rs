use std::fmt::{self, Display, Formatter};

use crate::ui::{
  atoms::{badge::Badge, id::Id, tag::Tags, title::Title},
  layout::Row,
  theme::Theme,
  utils,
};

/// Max display width for artifact titles in list rows.
const TITLE_PAD: usize = 35;

/// A single row in an artifact list, showing id, kind, title, tags, and optional archived badge.
pub struct ArtifactListRow<'a> {
  id: &'a str,
  is_archived: bool,
  kind: Option<&'a str>,
  kind_pad: usize,
  tags: &'a [String],
  theme: &'a Theme,
  title_text: &'a str,
}

impl<'a> ArtifactListRow<'a> {
  pub fn new(id: &'a str, title_text: &'a str, tags: &'a [String], theme: &'a Theme) -> Self {
    Self {
      id,
      title_text,
      tags,
      is_archived: false,
      kind: None,
      kind_pad: 0,
      theme,
    }
  }

  /// Marks this row as archived, applying dimmed styles and appending an `[archived]` badge.
  pub fn archived(mut self, v: bool) -> Self {
    self.is_archived = v;
    self
  }

  /// Sets the artifact kind (type) for the badge.
  pub fn kind(mut self, k: Option<&'a str>) -> Self {
    self.kind = k;
    self
  }

  /// Returns the rendered kind badge string for this row.
  pub fn kind_badge_string(&self) -> String {
    match self.kind {
      Some(k) => Badge::new(k, self.theme.artifact_list_kind).to_string(),
      None => String::new(),
    }
  }

  /// Returns the display width of the kind badge for this row.
  pub fn kind_badge_width(&self) -> usize {
    utils::display_width(&self.kind_badge_string())
  }

  /// Sets minimum display width for the kind badge column.
  pub fn kind_pad(mut self, w: usize) -> Self {
    self.kind_pad = w;
    self
  }
}

impl Display for ArtifactListRow<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut row = Row::new().spacing(2);

    row = row.col(Id::new(self.id, self.theme));

    let kind_str = self.kind_badge_string();
    if self.kind_pad > 0 {
      let pad = self.kind_pad.saturating_sub(utils::display_width(&kind_str));
      row = row.col(format!("{kind_str}{}", " ".repeat(pad)));
    } else if !kind_str.is_empty() {
      row = row.col(kind_str);
    }

    let title_style = if self.is_archived {
      self.theme.artifact_list_title_archived
    } else {
      self.theme.artifact_list_title
    };
    row = row.col(
      Title::new(self.title_text, title_style)
        .max_width(TITLE_PAD)
        .pad_to(TITLE_PAD),
    );

    let tag_style = if self.is_archived {
      self.theme.artifact_list_tag_archived
    } else {
      self.theme.tag
    };
    if !self.tags.is_empty() {
      row = row.col(Tags::new(self.tags, tag_style));
    }

    if self.is_archived {
      row = row.col(Badge::new("[archived]", self.theme.artifact_list_archived_badge));
    }

    write!(f, "{row}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  #[test]
  fn it_omits_kind_column_when_pad_is_zero() {
    yansi::disable();
    let theme = theme();
    let row = ArtifactListRow::new("aaaaaaaa", "no kind", &[], &theme);

    assert_eq!(row.kind_badge_width(), 0, "width should be zero with no kind");
  }

  #[test]
  fn it_renders_archived_row_with_badge() {
    yansi::disable();
    let theme = theme();
    let tags = vec!["spec".to_string(), "backend".to_string()];
    let row = ArtifactListRow::new("pkzqwrnd", "auth-spec", &tags, &theme).archived(true);
    let rendered = row.to_string();

    assert!(rendered.contains("pkzqwrnd"));
    assert!(rendered.contains("auth-spec"));
    assert!(rendered.contains("#spec"));
    assert!(rendered.contains("#backend"));
    assert!(rendered.contains("[archived]"));
  }

  #[test]
  fn it_renders_kind_badge() {
    yansi::disable();
    let theme = theme();
    let row = ArtifactListRow::new("abcdefgh", "my-spec", &[], &theme).kind(Some("spec"));
    let rendered = row.to_string();

    assert!(rendered.contains("spec"));
    assert!(rendered.contains("my-spec"));
  }

  #[test]
  fn it_renders_normal_row_with_id_title_and_tags() {
    yansi::disable();
    let theme = theme();
    let tags = vec!["schema".to_string()];
    let row = ArtifactListRow::new("fsahdqlt", "probe-schema-v2", &tags, &theme);
    let rendered = row.to_string();

    assert!(rendered.contains("fsahdqlt"));
    assert!(rendered.contains("probe-schema-v2"));
    assert!(rendered.contains("#schema"));
    assert!(!rendered.contains("[archived]"));
  }

  #[test]
  fn it_renders_row_with_multiple_tags() {
    yansi::disable();
    let theme = theme();
    let tags = vec!["spec".to_string(), "backend".to_string(), "v2".to_string()];
    let row = ArtifactListRow::new("abcdefgh", "my-artifact", &tags, &theme);
    let rendered = row.to_string();

    assert!(rendered.contains("#spec"));
    assert!(rendered.contains("#backend"));
    assert!(rendered.contains("#v2"));
    assert!(rendered.contains("#spec  #backend  #v2"));
  }

  #[test]
  fn it_renders_row_with_no_tags() {
    yansi::disable();
    let theme = theme();
    let row = ArtifactListRow::new("abcdefgh", "bare-artifact", &[], &theme);
    let rendered = row.to_string();

    assert!(rendered.contains("abcdefgh"));
    assert!(rendered.contains("bare-artifact"));
    assert!(!rendered.contains('#'));
  }

  #[test]
  fn it_reserves_kind_slot_when_padded() {
    yansi::disable();
    let theme = theme();
    let with_kind = ArtifactListRow::new("aaaaaaaa", "has kind", &[], &theme)
      .kind(Some("spec"))
      .kind_pad(4);
    let without_kind = ArtifactListRow::new("bbbbbbbb", "no kind", &[], &theme).kind_pad(4);

    let out_with = with_kind.to_string();
    let out_without = without_kind.to_string();

    assert!(out_with.contains("spec"), "should contain kind badge");

    let title_pos_with = out_with.find("has kind").unwrap();
    let title_pos_without = out_without.find("no kind").unwrap();
    assert_eq!(
      title_pos_with, title_pos_without,
      "titles should align when kind_pad is set"
    );
  }
}
