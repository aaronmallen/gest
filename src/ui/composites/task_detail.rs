use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{id::Id, label::Label, separator::Separator, tag::Tags, value::Value},
  composites::status_badge::StatusBadge,
  markdown,
  theme::Theme,
  utils,
};

/// Spacing between label and value columns.
const GAP: &str = "  ";

/// Indentation prefix for detail rows.
const INDENT: &str = "  ";

/// Renders the full detail view for a single task, including status, priority, links, tags, and optional body.
pub struct TaskDetail<'a> {
  assigned: Option<&'a str>,
  body: Option<&'a str>,
  id: &'a str,
  links: Vec<(&'a str, &'a str)>,
  phase: Option<(u32, Option<&'a str>)>,
  priority: Option<u8>,
  status: &'a str,
  tags: &'a [String],
  theme: &'a Theme,
  title: &'a str,
}

impl<'a> TaskDetail<'a> {
  pub fn new(id: &'a str, title: &'a str, status: &'a str, theme: &'a Theme) -> Self {
    Self {
      id,
      title,
      status,
      priority: None,
      phase: None,
      assigned: None,
      tags: &[],
      links: Vec::new(),
      body: None,
      theme,
    }
  }

  /// Sets the assignee for this task.
  pub fn assigned(mut self, a: Option<&'a str>) -> Self {
    self.assigned = a;
    self
  }

  /// Sets optional markdown body content to render in a description section.
  pub fn body(mut self, b: Option<&'a str>) -> Self {
    self.body = b;
    self
  }

  /// Sets task relationship links as `(relation, id)` pairs (e.g. `("blocked-by", "abc123")`).
  pub fn links(mut self, l: Vec<(&'a str, &'a str)>) -> Self {
    self.links = l;
    self
  }

  /// Sets the iteration phase as `(number, optional_name)`.
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
    let tw = utils::terminal_width() as usize;
    tw.saturating_sub(INDENT.len())
  }

  fn label(&self, text: &str, width: usize) -> Label {
    Label::new(text, self.theme.task_detail_label).pad_to(width)
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
      Some(n) => format!("{}  {}  {}", number, "\u{00B7}".paint(self.theme.muted), n),
      None => number.to_string(),
    }
  }

  fn status_badge(&self) -> StatusBadge<'_> {
    StatusBadge::new(self.status, self.theme)
  }
}

impl Display for TaskDetail<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let lw = self.label_width();

    writeln!(f, "{}", Id::new(self.id, self.theme))?;

    writeln!(f)?;

    writeln!(
      f,
      "{INDENT}{}{GAP}{}",
      self.label("title", lw),
      Value::new(self.title, self.theme.task_detail_title),
    )?;

    writeln!(f, "{INDENT}{}{GAP}{}", self.label("status", lw), self.status_badge(),)?;

    if let Some(p) = self.priority {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("priority", lw),
        Value::new(format!("P{p}"), self.theme.task_detail_value),
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
        Value::new(assigned, self.theme.task_detail_value),
      )?;
    }

    if !self.tags.is_empty() {
      writeln!(
        f,
        "{INDENT}{}{GAP}{}",
        self.label("tags", lw),
        Tags::new(self.tags, self.theme.tag),
      )?;
    }

    if !self.links.is_empty() {
      for (i, (relation, id)) in self.links.iter().enumerate() {
        let label = if i == 0 {
          self.label("links", lw)
        } else {
          Label::new("", self.theme.task_detail_label).pad_to(lw)
        };
        writeln!(
          f,
          "{INDENT}{label}{GAP}{} {}",
          relation.paint(self.theme.indicator_blocked_by_label),
          Id::new(id, self.theme),
        )?;
      }
    }

    if let Some(body) = self.body {
      writeln!(f)?;
      let sep = Separator::labeled("description", self.theme.task_detail_separator);
      writeln!(f, "{INDENT}{sep}")?;
      writeln!(f)?;

      let rendered = markdown::render(body, self.theme, self.body_width());
      for line in rendered.lines() {
        writeln!(f, "{INDENT}{line}")?;
      }

      writeln!(f)?;
      let rule = Separator::rule(self.theme.task_detail_separator);
      write!(f, "{INDENT}{rule}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(detail: &TaskDetail) -> String {
    yansi::disable();
    let out = detail.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_aligns_labels() {
    let t = theme();
    let tags = vec!["core".to_string()];
    let detail = TaskDetail::new("nfkbqmrx", "openai adapter", "in-progress", &t)
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
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "in-progress", &t);
    let out = render(&detail);
    assert!(out.contains('\u{25D0}'), "should contain in-progress icon");
  }

  #[test]
  fn it_omits_description_when_none() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t);
    let out = render(&detail);
    assert!(!out.contains("description"), "should not contain description header");
  }

  #[test]
  fn it_renders_assigned() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).assigned(Some("claude-code"));
    let out = render(&detail);
    assert!(out.contains("claude-code"));
  }

  #[test]
  fn it_renders_description_section() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).body(Some("## heading\n\nSome body text."));
    let out = render(&detail);
    assert!(out.contains("description"), "should contain section header");
    assert!(out.contains("heading"), "should contain rendered heading");
    assert!(out.contains("Some body text."), "should contain body text");
  }

  #[test]
  fn it_renders_full_detail() {
    let t = theme();
    let tags = vec!["adapter".to_string(), "core".to_string()];
    let detail = TaskDetail::new("nfkbqmrx", "openai streaming adapter", "in-progress", &t)
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
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "openai streaming adapter", "in-progress", &t);
    let out = render(&detail);
    let first_line = out.lines().next().unwrap();
    assert!(
      first_line.contains("nf") && first_line.contains("kbqmrx"),
      "first line should contain id parts, got: {first_line}",
    );
  }

  #[test]
  fn it_renders_links() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).links(vec![("blocked-by", "hpvrlbme")]);
    let out = render(&detail);
    assert!(out.contains("blocked-by"));
    assert!(out.contains("hpvrlbme"));
  }

  #[test]
  fn it_renders_multiple_links() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t)
      .links(vec![("blocked-by", "hpvrlbme"), ("blocks", "abcd1234")]);
    let out = render(&detail);
    assert!(out.contains("blocked-by"));
    assert!(out.contains("hpvrlbme"));
    assert!(out.contains("blocks"));
    assert!(out.contains("abcd1234"));
  }

  #[test]
  fn it_renders_phase_with_name() {
    let t = theme();
    let detail =
      TaskDetail::new("nfkbqmrx", "adapter", "in-progress", &t).phase(Some((2, Some("core implementation"))));
    let out = render(&detail);
    assert!(out.contains("2"), "should contain phase number");
    assert!(out.contains("core implementation"), "should contain phase name");
    assert!(out.contains("\u{00B7}"), "should contain middle dot separator");
  }

  #[test]
  fn it_renders_phase_without_name() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).phase(Some((3, None)));
    let out = render(&detail);
    assert!(out.contains('3'));
    assert!(!out.contains("\u{00B7}"));
  }

  #[test]
  fn it_renders_priority() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).priority(Some(1));
    let out = render(&detail);
    assert!(out.contains("P1"));
  }

  #[test]
  fn it_renders_tags() {
    let t = theme();
    let tags = vec!["adapter".to_string(), "core".to_string()];
    let detail = TaskDetail::new("nfkbqmrx", "adapter", "open", &t).tags(&tags);
    let out = render(&detail);
    assert!(out.contains("#adapter"));
    assert!(out.contains("#core"));
  }

  #[test]
  fn it_renders_title_and_status() {
    let t = theme();
    let detail = TaskDetail::new("nfkbqmrx", "openai streaming adapter", "in-progress", &t);
    let out = render(&detail);
    assert!(out.contains("openai streaming adapter"));
    assert!(out.contains("in progress"));
  }
}
