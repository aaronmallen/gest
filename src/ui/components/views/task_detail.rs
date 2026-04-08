//! Single-task detail view composing status, priority, links, tags, and a markdown body section.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  components::{
    atoms::{Id, Label, Separator, Tag, Value},
    molecules::{StatusBadge, row},
  },
  markdown,
};

/// Spacing between label and value columns.
const GAP: &str = "  ";

/// Indentation prefix for detail rows.
const INDENT: &str = "  ";

/// Renders the full detail view for a single task, including status, priority, links, tags, and optional body.
pub struct Component<'a> {
  assigned: Option<&'a str>,
  body: Option<&'a str>,
  id: &'a str,
  id_prefix_len: usize,
  links: Vec<(&'a str, &'a str)>,
  phase: Option<(u32, Option<&'a str>)>,
  priority: Option<u8>,
  status: &'a str,
  tags: &'a [String],
  title: &'a str,
}

impl<'a> Component<'a> {
  /// Create a detail view for a task with the given id, title, and status.
  pub fn new(id: &'a str, title: &'a str, status: &'a str) -> Self {
    Self {
      assigned: None,
      body: None,
      id,
      id_prefix_len: 2,
      links: Vec::new(),
      phase: None,
      priority: None,
      status,
      tags: &[],
      title,
    }
  }

  /// Sets the assignee for this task.
  #[cfg(test)]
  pub fn assigned(mut self, a: Option<&'a str>) -> Self {
    self.assigned = a;
    self
  }

  /// Sets optional markdown body content to render in a description section.
  pub fn body(mut self, b: Option<&'a str>) -> Self {
    self.body = b;
    self
  }

  /// Sets the highlighted prefix length used for the heading and link IDs.
  pub fn id_prefix_len(mut self, len: usize) -> Self {
    self.id_prefix_len = len;
    self
  }

  /// Sets task relationship links as `(relation, id)` pairs (e.g. `("blocked-by", "abc123")`).
  #[cfg(test)]
  pub fn links(mut self, l: Vec<(&'a str, &'a str)>) -> Self {
    self.links = l;
    self
  }

  /// Sets the iteration phase as `(number, optional_name)`.
  #[cfg(test)]
  pub fn phase(mut self, phase: Option<(u32, Option<&'a str>)>) -> Self {
    self.phase = phase;
    self
  }

  /// Sets the task priority level.
  pub fn priority(mut self, p: Option<u8>) -> Self {
    self.priority = p;
    self
  }

  /// Sets the tags to display for this task.
  pub fn tags(mut self, t: &'a [String]) -> Self {
    self.tags = t;
    self
  }

  fn body_width(&self) -> usize {
    let tw = row::terminal_width() as usize;
    tw.saturating_sub(INDENT.len())
  }

  fn label(&self, text: &str, width: usize) -> Label {
    Label::new(text, *crate::ui::style::global().task_detail_label()).pad_to(width)
  }

  fn label_width(&self) -> usize {
    let mut labels: Vec<&str> = vec!["title", "status"];

    if self.priority.is_some() {
      labels.push("priority");
    }
    if self.phase.is_some() {
      labels.push("phase");
    }
    if self.assigned.is_some() {
      labels.push("assigned");
    }
    if !self.tags.is_empty() {
      labels.push("tags");
    }
    if !self.links.is_empty() {
      labels.push("links");
    }

    labels.iter().map(|l| l.len()).max().unwrap_or(0)
  }

  fn phase_value(&self, number: u32, name: Option<&str>) -> String {
    match name {
      Some(n) => format!(
        "{}  {}  {}",
        number,
        "\u{00B7}".paint(*crate::ui::style::global().muted()),
        n
      ),
      None => number.to_string(),
    }
  }

  fn status_badge(&self) -> StatusBadge<'_> {
    StatusBadge::new(self.status)
  }
}

impl Display for Component<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    let lw = self.label_width();

    writeln!(f, "{}", Id::new(self.id).prefix_len(self.id_prefix_len))?;

    writeln!(f)?;

    writeln!(
      f,
      "{INDENT}{}{GAP}{}",
      self.label("title", lw),
      Value::new(self.title, *theme.task_detail_title()),
    )?;

    writeln!(f, "{INDENT}{}{GAP}{}", self.label("status", lw), self.status_badge(),)?;

    if let Some(p) = self.priority {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("priority", lw),
        Value::new(format!("P{p}"), *theme.task_detail_value()),
      )?;
    }

    if let Some((number, name)) = self.phase {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("phase", lw),
        self.phase_value(number, name),
      )?;
    }

    if let Some(assigned) = self.assigned {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("assigned", lw),
        Value::new(assigned, *theme.task_detail_value()),
      )?;
    }

    if !self.tags.is_empty() {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("tags", lw),
        Tag::new(self.tags.to_vec(), *theme.tag()),
      )?;
    }

    if !self.links.is_empty() {
      for (i, (relation, id)) in self.links.iter().enumerate() {
        let label = if i == 0 {
          self.label("links", lw)
        } else {
          Label::new("", *theme.task_detail_label()).pad_to(lw)
        };
        writeln!(
          f,
          "{INDENT}{label}{GAP}{} {}",
          relation.paint(*theme.indicator_blocked_by_label()),
          Id::new(id).prefix_len(self.id_prefix_len),
        )?;
      }
    }

    if let Some(body) = self.body {
      writeln!(f)?;
      let sep = Separator::labeled("description", *theme.task_detail_separator());
      writeln!(f, "{INDENT}{sep}")?;
      writeln!(f)?;

      let rendered = markdown::render(body, self.body_width());
      for line in rendered.lines() {
        writeln!(f, "{INDENT}{line}")?;
      }

      writeln!(f)?;
      let rule = Separator::rule(*theme.task_detail_separator());
      write!(f, "{INDENT}{rule}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(detail: &Component) -> String {
    yansi::disable();
    let out = detail.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_aligns_labels() {
    let tags = vec!["core".to_string()];
    let detail = Component::new("nfkbqmrx", "openai adapter", "in-progress")
      .priority(Some(1))
      .assigned(Some("claude-code"))
      .tags(&tags);
    let out = render(&detail);

    assert!(out.contains("title"), "should contain title label");
    assert!(out.contains("status"), "should contain status label");
    assert!(out.contains("priority"), "should contain priority label");
    assert!(out.contains("assigned"), "should contain assigned label");
    assert!(out.contains("tags"), "should contain tags label");

    assert_eq!(detail.label_width(), 8);
  }

  #[test]
  fn it_includes_status_icon() {
    let detail = Component::new("nfkbqmrx", "adapter", "in-progress");
    let out = render(&detail);

    assert!(out.contains('\u{25D0}'), "should contain in-progress icon");
  }

  #[test]
  fn it_omits_description_when_none() {
    let detail = Component::new("nfkbqmrx", "adapter", "open");
    let out = render(&detail);

    assert!(!out.contains("description"), "should not contain description header");
  }

  #[test]
  fn it_renders_assigned() {
    let detail = Component::new("nfkbqmrx", "adapter", "open").assigned(Some("claude-code"));
    let out = render(&detail);

    assert!(out.contains("claude-code"));
  }

  #[test]
  fn it_renders_description_section() {
    let detail = Component::new("nfkbqmrx", "adapter", "open").body(Some("## heading\n\nSome body text."));
    let out = render(&detail);

    assert!(out.contains("description"), "should contain section header");
    assert!(out.contains("heading"), "should contain rendered heading");
    assert!(out.contains("Some body text."), "should contain body text");
  }

  #[test]
  fn it_renders_full_detail() {
    let tags = vec!["adapter".to_string(), "core".to_string()];
    let detail = Component::new("nfkbqmrx", "openai streaming adapter", "in-progress")
      .priority(Some(1))
      .phase(Some((2, Some("core implementation"))))
      .assigned(Some("claude-code"))
      .tags(&tags)
      .links(vec![("blocked-by", "hpvrlbme")])
      .body(Some("## openai streaming adapter\n\nImplement the adapter."));

    let out = render(&detail);

    assert!(!out.is_empty());
  }

  #[test]
  fn it_renders_id_on_first_line() {
    let detail = Component::new("nfkbqmrx", "openai streaming adapter", "in-progress");
    let out = render(&detail);
    let first_line = out.lines().next().unwrap();

    assert!(
      first_line.contains("nf") && first_line.contains("kbqmrx"),
      "first line should contain id parts, got: {first_line}",
    );
  }

  #[test]
  fn it_renders_links() {
    let detail = Component::new("nfkbqmrx", "adapter", "open").links(vec![("blocked-by", "hpvrlbme")]);
    let out = render(&detail);

    assert!(out.contains("blocked-by"));
    assert!(out.contains("hpvrlbme"));
  }

  #[test]
  fn it_renders_multiple_links() {
    let detail =
      Component::new("nfkbqmrx", "adapter", "open").links(vec![("blocked-by", "hpvrlbme"), ("blocks", "abcd1234")]);
    let out = render(&detail);

    assert!(out.contains("blocked-by"));
    assert!(out.contains("hpvrlbme"));
    assert!(out.contains("blocks"));
    assert!(out.contains("abcd1234"));
  }

  #[test]
  fn it_renders_phase_with_name() {
    let detail = Component::new("nfkbqmrx", "adapter", "in-progress").phase(Some((2, Some("core implementation"))));
    let out = render(&detail);

    assert!(out.contains("2"), "should contain phase number");
    assert!(out.contains("core implementation"), "should contain phase name");
    assert!(out.contains("\u{00B7}"), "should contain middle dot separator");
  }

  #[test]
  fn it_renders_phase_without_name() {
    let detail = Component::new("nfkbqmrx", "adapter", "open").phase(Some((3, None)));
    let out = render(&detail);

    assert!(out.contains('3'));
    assert!(!out.contains("\u{00B7}"));
  }

  #[test]
  fn it_renders_priority() {
    let detail = Component::new("nfkbqmrx", "adapter", "open").priority(Some(1));
    let out = render(&detail);

    assert!(out.contains("P1"));
  }

  #[test]
  fn it_renders_tags() {
    let tags = vec!["adapter".to_string(), "core".to_string()];
    let detail = Component::new("nfkbqmrx", "adapter", "open").tags(&tags);
    let out = render(&detail);

    assert!(out.contains("#adapter"));
    assert!(out.contains("#core"));
  }

  #[test]
  fn it_renders_title_and_status() {
    let detail = Component::new("nfkbqmrx", "openai streaming adapter", "in-progress");
    let out = render(&detail);

    assert!(out.contains("openai streaming adapter"));
    assert!(out.contains("in progress"));
  }
}
