//! Inline blocked/blocking relationship indicators molecule for task rows.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::components::atoms::{Icon, Id};

/// Renders inline blocked/blocking status indicators for a task.
pub struct Component<'a> {
  blocked_by: Vec<&'a str>,
  is_blocking: bool,
  prefix_len: usize,
}

impl<'a> Component<'a> {
  /// Create an empty indicators block with no blocking relationships set.
  pub fn new() -> Self {
    Self {
      blocked_by: Vec::new(),
      is_blocking: false,
      prefix_len: 2,
    }
  }

  /// Sets the IDs of tasks blocking this one.
  pub fn blocked_by(mut self, ids: Vec<&'a str>) -> Self {
    self.blocked_by = ids;
    self
  }

  /// Marks this task as blocking other tasks.
  pub fn blocking(mut self, v: bool) -> Self {
    self.is_blocking = v;
    self
  }

  /// Set the highlighted prefix length passed to rendered [`Id`]s.
  pub fn prefix_len(mut self, len: usize) -> Self {
    self.prefix_len = len;
    self
  }
}

impl Display for Component<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    let mut parts = Vec::new();

    if self.is_blocking {
      let icon = Icon::blocking();
      let label = "blocking".paint(*theme.indicator_blocking());
      parts.push(format!("{icon} {label}"));
    }

    for id in &self.blocked_by {
      let label = "blocked-by".paint(*theme.indicator_blocked_by_label());
      let id = Id::new(id).prefix_len(self.prefix_len);
      parts.push(format!("{label} {id}"));
    }

    write!(f, "{}", parts.join("  "))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(indicators: &Component) -> String {
    yansi::disable();
    let out = indicators.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_renders_blocked_by() {
    let ind = Component::new().blocked_by(vec!["hpvrlbme"]);
    let out = render(&ind);

    assert!(out.contains("blocked-by"));
    assert!(out.contains("hpvrlbme"));
  }

  #[test]
  fn it_renders_blocking_only() {
    let ind = Component::new().blocking(true);
    let out = render(&ind);

    assert!(out.contains("! blocking"));
  }

  #[test]
  fn it_renders_empty_for_no_indicators() {
    let ind = Component::new();
    let out = render(&ind);

    assert_eq!(out, "");
  }
}
