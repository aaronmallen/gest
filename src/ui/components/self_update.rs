use std::io;

use yansi::Paint;

use crate::ui::theme::Theme;

/// Message displayed when the user is already on the target version.
///
/// Produces output like: `OK Already on v0.1.0`
pub struct AlreadyOnVersion {
  version: String,
}

impl AlreadyOnVersion {
  pub fn new(version: &str) -> Self {
    Self {
      version: version.to_string(),
    }
  }

  /// Write the already-on-version message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(w, "{} Already on v{}", "OK".paint(theme.success), self.version)
  }
}

crate::ui::macros::impl_display_via_write_to!(AlreadyOnVersion, theme);

/// Message displaying the available update transition.
///
/// Produces output like: `Update available: v0.0.1 -> v0.1.0`
pub struct UpdateAvailable {
  current: String,
  target: String,
}

impl UpdateAvailable {
  pub fn new(current: &str, target: &str) -> Self {
    Self {
      current: current.to_string(),
      target: target.to_string(),
    }
  }

  /// Write the update-available message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "Update available: v{} -> v{}", self.current, self.target)
  }
}

crate::ui::macros::impl_display_via_write_to!(UpdateAvailable);

/// Prompt asking the user to confirm the update.
///
/// Produces output like: `Do you want to continue? [y/N] `
pub struct UpdatePrompt;

impl UpdatePrompt {
  /// Write the update prompt to the given writer (no trailing newline).
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    write!(w, "Do you want to continue? [y/N] ")
  }
}

/// Message displayed when the user cancels the update.
///
/// Produces output: `Update cancelled.`
pub struct UpdateCancelled;

impl UpdateCancelled {
  /// Write the update-cancelled message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
    writeln!(w, "Update cancelled.")
  }
}

crate::ui::macros::impl_display_via_write_to!(UpdateCancelled);

/// Message displayed after a successful update.
///
/// Produces output like: `OK Updated to v0.1.0`
pub struct UpdateComplete {
  version: String,
}

impl UpdateComplete {
  pub fn new(version: &str) -> Self {
    Self {
      version: version.to_string(),
    }
  }

  /// Write the update-complete message to the given writer.
  pub fn write_to(&self, w: &mut impl io::Write, theme: &Theme) -> io::Result<()> {
    writeln!(w, "{} Updated to v{}", "OK".paint(theme.success), self.version)
  }
}

crate::ui::macros::impl_display_via_write_to!(UpdateComplete, theme);

#[cfg(test)]
mod tests {
  use super::*;

  mod already_on_version {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = AlreadyOnVersion::new("1.2.3");
        let display = msg.to_string();
        assert!(display.contains("Already on v1.2.3"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_ok_message_with_version() {
        let msg = AlreadyOnVersion::new("1.2.3");
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("OK"), "Should contain 'OK'");
        assert!(output.contains("Already on v1.2.3"), "Should contain version");
      }
    }
  }

  mod update_available {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = UpdateAvailable::new("0.0.1", "0.1.0");
        let display = msg.to_string();
        assert!(display.contains("Update available: v0.0.1 -> v0.1.0"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_version_transition() {
        let msg = UpdateAvailable::new("0.0.1", "0.1.0");
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Update available: v0.0.1 -> v0.1.0"));
      }
    }
  }

  mod update_cancelled {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = UpdateCancelled;
        let display = msg.to_string();
        assert!(display.contains("Update cancelled."));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_cancelled_message() {
        let msg = UpdateCancelled;
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Update cancelled."));
      }
    }
  }

  mod update_complete {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_delegates_to_write_to() {
        let msg = UpdateComplete::new("0.1.0");
        let display = msg.to_string();
        assert!(display.contains("Updated to v0.1.0"));
      }
    }

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_ok_message_with_version() {
        let msg = UpdateComplete::new("0.1.0");
        let theme = Theme::default();
        let mut buf = Vec::new();
        msg.write_to(&mut buf, &theme).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("OK"), "Should contain 'OK'");
        assert!(output.contains("Updated to v0.1.0"), "Should contain version");
      }
    }
  }

  mod update_prompt {
    use super::*;

    mod write_to {
      use super::*;

      #[test]
      fn it_writes_prompt_without_newline() {
        let msg = UpdatePrompt;
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Do you want to continue? [y/N]"));
        assert!(!output.ends_with('\n'), "Should not end with newline");
      }
    }
  }
}
