use std::fmt::{self, Display, Formatter};

use yansi::Paint;

use crate::ui::{
  atoms::{label::Label, value::Value},
  composites::success_message::SuccessMessage,
  theme::Theme,
};

/// Renders the resolved configuration summary (paths, settings, color overrides).
pub struct ConfigView<'a> {
  data_dir: &'a str,
  global_config: Option<&'a str>,
  has_color_overrides: bool,
  log_level: &'a str,
  project_config: Option<&'a str>,
  theme: &'a Theme,
}

impl<'a> ConfigView<'a> {
  pub fn new(data_dir: &'a str, log_level: &'a str, theme: &'a Theme) -> Self {
    Self {
      global_config: None,
      project_config: None,
      data_dir,
      log_level,
      has_color_overrides: false,
      theme,
    }
  }

  /// Sets the global config file path to display.
  pub fn global_config(mut self, path: &'a str) -> Self {
    self.global_config = Some(path);
    self
  }

  /// Indicates whether custom color overrides are active.
  pub fn has_color_overrides(mut self, has: bool) -> Self {
    self.has_color_overrides = has;
    self
  }

  /// Sets the project-local config file path to display.
  pub fn project_config(mut self, path: &'a str) -> Self {
    self.project_config = Some(path);
    self
  }
}

impl Display for ConfigView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}", "configuration".paint(self.theme.config_heading),)?;

    let path_label_width = 7;
    writeln!(f)?;
    writeln!(
      f,
      "  {}  {}",
      Label::new("global", self.theme.config_label).pad_to(path_label_width),
      Value::new(self.global_config.unwrap_or("(none)"), self.theme.config_value,),
    )?;
    writeln!(
      f,
      "  {}  {}",
      Label::new("project", self.theme.config_label).pad_to(path_label_width),
      Value::new(self.project_config.unwrap_or("(none)"), self.theme.config_value,),
    )?;

    let setting_label_width = 9;
    writeln!(f)?;
    writeln!(
      f,
      "  {}  {}",
      Label::new("data_dir", self.theme.config_label).pad_to(setting_label_width),
      Value::new(self.data_dir, self.theme.config_value),
    )?;
    writeln!(
      f,
      "  {}  {}",
      Label::new("log_level", self.theme.config_label).pad_to(setting_label_width),
      Value::new(self.log_level, self.theme.config_value),
    )?;

    writeln!(f)?;
    if self.has_color_overrides {
      write!(
        f,
        "  {}  {}",
        Label::new("colors", self.theme.config_label).pad_to(setting_label_width),
        Value::new("(custom overrides present)", self.theme.config_value),
      )?;
    } else {
      write!(
        f,
        "  {}  {}",
        Label::new("colors", self.theme.config_label).pad_to(setting_label_width),
        "(no overrides \u{2014} using defaults)".paint(self.theme.config_no_overrides),
      )?;
    }

    Ok(())
  }
}

/// Renders the post-initialization success message with getting-started hints.
pub struct InitView<'a> {
  config_path: Option<&'a str>,
  data_dir: &'a str,
  theme: &'a Theme,
}

impl<'a> InitView<'a> {
  pub fn new(data_dir: &'a str, config_path: Option<&'a str>, theme: &'a Theme) -> Self {
    Self {
      data_dir,
      config_path,
      theme,
    }
  }
}

impl Display for InitView<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut msg = SuccessMessage::new("initialized gest", self.theme).field("data dir", self.data_dir);
    if let Some(config) = self.config_path {
      msg = msg.field("config", config);
    }
    write!(f, "{msg}")?;

    writeln!(f)?;
    writeln!(f)?;
    writeln!(f, "  {}", "get started".paint(self.theme.init_section),)?;
    for cmd in [
      "gest task create \"my first task\"",
      "gest artifact create --file spec.md",
      "gest iteration create \"sprint 1\"",
    ] {
      writeln!(
        f,
        "    {} {}",
        "$".paint(self.theme.init_command_prefix),
        cmd.paint(self.theme.init_value),
      )?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    yansi::disable();
    Theme::default()
  }

  mod config_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_file_paths() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme)
          .global_config("~/.config/gest/config.toml")
          .project_config(".gest/config.toml");
        let rendered = format!("{view}");

        assert!(rendered.contains("global"));
        assert!(rendered.contains("~/.config/gest/config.toml"));
        assert!(rendered.contains("project"));
        assert!(rendered.contains(".gest/config.toml"));
      }

      #[test]
      fn it_renders_heading() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains("configuration"));
      }

      #[test]
      fn it_renders_settings() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains("data_dir"));
        assert!(rendered.contains(".gest/"));
        assert!(rendered.contains("log_level"));
        assert!(rendered.contains("warn"));
      }

      #[test]
      fn it_shows_no_overrides_by_default() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains("no overrides"));
        assert!(rendered.contains("using defaults"));
      }

      #[test]
      fn it_shows_none_for_missing_paths() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains("(none)"));
      }

      #[test]
      fn it_shows_overrides_when_present() {
        let theme = theme();
        let view = ConfigView::new(".gest/", "warn", &theme).has_color_overrides(true);
        let rendered = format!("{view}");

        assert!(rendered.contains("custom overrides present"));
        assert!(!rendered.contains("no overrides"));
      }
    }
  }

  mod init_view {
    use super::*;

    mod display {
      use super::*;

      #[test]
      fn it_renders_get_started_section() {
        let theme = theme();
        let view = InitView::new(".gest/", Some(".gest/config.toml"), &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains("get started"));
        assert!(rendered.contains("$ gest task create"));
        assert!(rendered.contains("$ gest artifact create --file spec.md"));
        assert!(rendered.contains("$ gest iteration create"));
      }

      #[test]
      fn it_renders_success_and_paths_for_local_mode() {
        let theme = theme();
        let view = InitView::new(".gest/", Some(".gest/config.toml"), &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains('\u{2713}'), "expected check icon");
        assert!(rendered.contains("initialized gest"));
        assert!(rendered.contains(".gest/"));
        assert!(rendered.contains(".gest/config.toml"));
      }

      #[test]
      fn it_renders_success_without_config_for_global_mode() {
        let theme = theme();
        let view = InitView::new("/home/user/.local/share/gest/abc123", None, &theme);
        let rendered = format!("{view}");

        assert!(rendered.contains('\u{2713}'), "expected check icon");
        assert!(rendered.contains("initialized gest"));
        assert!(rendered.contains("/home/user/.local/share/gest/abc123"));
        assert!(!rendered.contains("config"));
      }
    }
  }
}
