//! Update-available notice molecule displayed beneath the startup banner.

use std::fmt::{self, Display, Formatter};

use yansi::Paint;

/// Renders the "a newer version is available" notice shown beneath the banner
/// when a GitHub release check turns up a newer version of `gest`.
pub struct Component {
  new_version: String,
}

impl Component {
  /// Create an update notice for the given new version string.
  pub fn new(new_version: String) -> Self {
    Self {
      new_version,
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();

    write!(
      f,
      "{} {}",
      "a newer version is available".paint(*theme.banner_update_message()),
      self.new_version.paint(*theme.banner_update_version())
    )?;
    write!(
      f,
      "\n{}{}{}",
      "run ".paint(*theme.banner_update_hint()),
      "gest self-update".paint(*theme.banner_update_command()),
      " to upgrade".paint(*theme.banner_update_hint())
    )?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_renders_the_new_version() {
    let notice = Component::new("9.9.9".to_string());

    let output = notice.to_string();

    assert!(output.contains("9.9.9"));
  }

  #[test]
  fn it_renders_the_update_message() {
    let notice = Component::new("9.9.9".to_string());

    let output = notice.to_string();

    assert!(output.contains("a newer version is available"));
  }

  #[test]
  fn it_renders_the_upgrade_hint() {
    let notice = Component::new("9.9.9".to_string());

    let output = notice.to_string();

    assert!(output.contains("gest self-update"));
  }
}
