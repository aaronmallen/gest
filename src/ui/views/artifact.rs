use std::fmt::{self, Display, Formatter};

use crate::ui::{
  composites::{
    artifact_detail::ArtifactDetail, artifact_list_row::ArtifactListRow, grouped_list::GroupedList,
    success_message::SuccessMessage,
  },
  theme::Theme,
};

/// Renders a success message after creating an artifact.
pub struct ArtifactCreateView<'a> {
  id: &'a str,
  source: Option<&'a str>,
  theme: &'a Theme,
  title: &'a str,
}

impl<'a> ArtifactCreateView<'a> {
  pub fn new(id: &'a str, title: &'a str, theme: &'a Theme) -> Self {
    Self {
      id,
      title,
      source: None,
      theme,
    }
  }

  /// Sets the optional source file path shown in the confirmation.
  pub fn source(mut self, source: &'a str) -> Self {
    self.source = Some(source);
    self
  }
}

impl Display for ArtifactCreateView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("created artifact", self.theme)
      .id(self.id)
      .field("title", self.title);

    if let Some(src) = self.source {
      msg = msg.field("source", src);
    }

    write!(f, "{msg}")
  }
}

/// Renders the full detail page for a single artifact.
pub struct ArtifactDetailView<'a> {
  detail: ArtifactDetail<'a>,
}

impl<'a> ArtifactDetailView<'a> {
  pub fn new(detail: ArtifactDetail<'a>) -> Self {
    Self {
      detail,
    }
  }
}

impl Display for ArtifactDetailView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.detail)
  }
}

/// Data for a single row in the artifact list view.
pub struct ArtifactViewData {
  pub id: String,
  pub is_archived: bool,
  pub kind: Option<String>,
  pub tags: Vec<String>,
  pub title: String,
}

/// Renders a grouped list of artifacts with a summary header.
pub struct ArtifactListView<'a> {
  archived: usize,
  artifacts: Vec<ArtifactViewData>,
  theme: &'a Theme,
  total: usize,
}

impl<'a> ArtifactListView<'a> {
  pub fn new(total: usize, archived: usize, theme: &'a Theme) -> Self {
    Self {
      artifacts: Vec::new(),
      total,
      archived,
      theme,
    }
  }

  /// Appends multiple artifact data entries to the list.
  pub fn artifacts(mut self, items: impl IntoIterator<Item = ArtifactViewData>) -> Self {
    self.artifacts.extend(items);
    self
  }
}

impl Display for ArtifactListView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let summary = if self.archived > 0 {
      format!(
        "{} artifact{}  \u{00b7}  {} archived",
        self.total,
        if self.total == 1 { "" } else { "s" },
        self.archived,
      )
    } else {
      format!("{} artifact{}", self.total, if self.total == 1 { "" } else { "s" },)
    };

    let proto_rows: Vec<ArtifactListRow> = self
      .artifacts
      .iter()
      .map(|a| {
        ArtifactListRow::new(&a.id, &a.title, &a.tags, self.theme)
          .kind(a.kind.as_deref())
          .archived(a.is_archived)
      })
      .collect();

    let max_kind = proto_rows.iter().map(|r| r.kind_badge_width()).max().unwrap_or(0);

    let rows: Vec<String> = proto_rows
      .into_iter()
      .map(|r| r.kind_pad(max_kind).to_string())
      .collect();

    let list = GroupedList::new("artifacts", summary, self.theme).rows(rows);

    write!(f, "{list}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  #[test]
  fn it_delegates_detail_view_to_composite() {
    yansi::disable();
    let theme = theme();
    let detail = ArtifactDetail::new("fsahdqlt", "probe-schema-v2", &[], "2026-03-27", "2026-03-29", &theme);
    let view = ArtifactDetailView::new(detail);
    let output = view.to_string();

    assert!(output.contains("fsahdqlt"), "should contain id");
    assert!(output.contains("probe-schema-v2"), "should contain title");
  }

  #[test]
  fn it_omits_archived_segment_when_none() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(3, 0, &theme);
    let output = view.to_string();

    assert!(!output.contains("archived"), "should omit archived when zero");
    assert!(!output.contains('\u{00b7}'), "should omit dot when no archived");
  }

  #[test]
  fn it_renders_create_view_success_message() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactCreateView::new("nfkbqmrx", "probe-schema-v2", &theme).source("probe-schema-v2.md");
    let output = view.to_string();

    assert!(output.contains('\u{2713}'), "should contain check icon");
    assert!(output.contains("created artifact"), "should contain action");
    assert!(output.contains("title"), "should contain title label");
    assert!(output.contains("probe-schema-v2"), "should contain title value");
    assert!(output.contains("source"), "should contain source label");
    assert!(output.contains("probe-schema-v2.md"), "should contain source value");
  }

  #[test]
  fn it_renders_create_view_without_source() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactCreateView::new("abcdefgh", "my-artifact", &theme);
    let output = view.to_string();

    assert!(output.contains("created artifact"), "should contain action");
    assert!(output.contains("my-artifact"), "should contain title");
    assert!(!output.contains("source"), "should not contain source label");
  }

  #[test]
  fn it_renders_list_view_heading_and_summary() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(5, 2, &theme);
    let output = view.to_string();

    assert!(output.contains("artifacts"), "should contain heading");
    assert!(output.contains("5 artifacts"), "should contain total");
    assert!(output.contains("2 archived"), "should contain archived count");
    assert!(output.contains('\u{00b7}'), "should contain middle dot separator");
  }

  #[test]
  fn it_renders_list_view_rows() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(2, 0, &theme).artifacts(vec![
      ArtifactViewData {
        id: "abcdefgh".into(),
        title: "first-artifact".into(),
        kind: None,
        tags: vec!["spec".into()],
        is_archived: false,
      },
      ArtifactViewData {
        id: "ijklmnop".into(),
        title: "second-artifact".into(),
        kind: None,
        tags: vec![],
        is_archived: false,
      },
    ]);
    let output = view.to_string();

    assert!(output.contains("abcdefgh"), "should contain first row id");
    assert!(output.contains("first-artifact"), "should contain first row title");
    assert!(output.contains("ijklmnop"), "should contain second row id");
    assert!(output.contains("second-artifact"), "should contain second row title");
  }

  #[test]
  fn it_renders_list_view_with_singular_artifact() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(1, 0, &theme);
    let output = view.to_string();

    assert!(output.contains("1 artifact"), "should use singular");
    assert!(!output.contains("1 artifacts"), "should not use plural for count of 1");
    assert!(!output.contains("archived"), "should omit archived when zero");
  }

  #[test]
  fn it_renders_kind_in_list_rows() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(2, 0, &theme).artifacts(vec![
      ArtifactViewData {
        id: "abcdefgh".into(),
        title: "my-spec".into(),
        kind: Some("spec".into()),
        tags: vec![],
        is_archived: false,
      },
      ArtifactViewData {
        id: "ijklmnop".into(),
        title: "my-adr".into(),
        kind: Some("adr".into()),
        tags: vec![],
        is_archived: false,
      },
    ]);
    let output = view.to_string();

    assert!(output.contains("spec"), "should contain kind for first row");
    assert!(output.contains("adr"), "should contain kind for second row");
  }

  #[test]
  fn it_aligns_titles_with_mixed_kinds() {
    yansi::disable();
    let theme = theme();
    let view = ArtifactListView::new(2, 0, &theme).artifacts(vec![
      ArtifactViewData {
        id: "aaaaaaaa".into(),
        title: "has-kind".into(),
        kind: Some("spec".into()),
        tags: vec![],
        is_archived: false,
      },
      ArtifactViewData {
        id: "bbbbbbbb".into(),
        title: "no-kind".into(),
        kind: None,
        tags: vec![],
        is_archived: false,
      },
    ]);
    let output = view.to_string();
    let lines: Vec<&str> = output.lines().collect();

    let line_with = lines
      .iter()
      .find(|l| l.contains("has-kind"))
      .expect("should find has-kind line");
    let line_without = lines
      .iter()
      .find(|l| l.contains("no-kind"))
      .expect("should find no-kind line");

    let pos_with = line_with.find("has-kind").unwrap();
    let pos_without = line_without.find("no-kind").unwrap();

    assert_eq!(pos_with, pos_without, "titles should align when kinds differ");
  }
}
