use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{icon::Icon, id::Id},
  theme::Theme,
};

/// Renders inline blocked/blocking status indicators for a task.
pub struct Indicators<'a> {
  blocked_by: Vec<&'a str>,
  is_blocking: bool,
  theme: &'a Theme,
}

impl<'a> Indicators<'a> {
  pub fn new(theme: &'a Theme) -> Self {
    Self {
      is_blocking: false,
      blocked_by: Vec::new(),
      theme,
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
}

impl Display for Indicators<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut parts = Vec::new();

    if self.is_blocking {
      let icon = Icon::blocking(self.theme);
      let label = "blocking".paint(self.theme.indicator_blocking);
      parts.push(format!("{icon} {label}"));
    }

    for id in &self.blocked_by {
      let label = "blocked-by".paint(self.theme.indicator_blocked_by_label);
      let id = Id::new(id, self.theme);
      parts.push(format!("{label} {id}"));
    }

    write!(f, "{}", parts.join("  "))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  fn render(indicators: &Indicators) -> String {
    yansi::disable();
    let out = indicators.to_string();
    yansi::enable();
    out
  }

  #[test]
  fn it_renders_blocked_by_one_id() {
    let t = theme();
    let indicators = Indicators::new(&t).blocked_by(vec!["hpvrlbme"]);
    let output = render(&indicators);
    assert!(output.contains("blocked-by"));
    assert!(output.contains("hpvrlbme"));
  }

  #[test]
  fn it_renders_blocking_only() {
    let t = theme();
    let indicators = Indicators::new(&t).blocking(true);
    let output = render(&indicators);
    assert!(output.contains("! blocking"));
    assert!(!output.contains("blocked-by"));
  }

  #[test]
  fn it_renders_both_blocked_and_blocking() {
    let t = theme();
    let indicators = Indicators::new(&t).blocked_by(vec!["abc12345"]).blocking(true);
    let output = render(&indicators);
    assert!(output.contains("blocked-by"));
    assert!(output.contains("abc12345"));
    assert!(output.contains("! blocking"));
  }

  #[test]
  fn it_renders_empty_string_for_no_indicators() {
    let t = theme();
    let indicators = Indicators::new(&t);
    let output = render(&indicators);
    assert_eq!(output, "");
  }
}
