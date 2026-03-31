use std::fmt::{self, Display, Formatter};

use yansi::{Color, Paint, Style};

use crate::ui::theme::Theme;

/// ASCII block-art rows for the GEST logo.
const ASCII_ART: [&str; 6] = [
  " ██████╗ ███████╗███████╗████████╗",
  "██╔════╝ ██╔════╝██╔════╝╚══██╔══╝",
  "██║  ███╗█████╗  ███████╗   ██║",
  "██║   ██║██╔══╝  ╚════██║   ██║",
  "╚██████╔╝███████╗███████║   ██║",
  " ╚═════╝ ╚══════╝╚══════╝   ╚═╝",
];

const ROW_COUNT: usize = ASCII_ART.len();

/// Box-drawing characters rendered with the shadow color for depth effect.
const SHADOW_CHARS: &[char] = &['╗', '╚', '╝', '╔', '║'];

/// Renders the application startup banner with gradient ASCII art, version info, and optional update notice.
pub struct Banner<'a> {
  author: &'a str,
  date: &'a str,
  platform: &'a str,
  revision: &'a str,
  show_version: bool,
  theme: &'a Theme,
  update_version: Option<String>,
  version: &'a str,
}

impl<'a> Banner<'a> {
  pub fn new(
    version: &'a str,
    platform: &'a str,
    date: &'a str,
    revision: &'a str,
    author: &'a str,
    theme: &'a Theme,
  ) -> Self {
    Self {
      version,
      platform,
      date,
      revision,
      show_version: true,
      author,
      update_version: None,
      theme,
    }
  }

  /// Hide the version line from the banner output.
  pub fn hide_version(mut self) -> Self {
    self.show_version = false;
    self
  }

  /// Enables a notice that a newer version is available.
  pub fn update_version(mut self, version: impl Into<String>) -> Self {
    self.update_version = Some(version.into());
    self
  }
}

impl Display for Banner<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let start = fg_color(&self.theme.banner_gradient_start);
    let end = fg_color(&self.theme.banner_gradient_end);
    let shadow_color = fg_color(&self.theme.banner_shadow);

    for (i, row) in ASCII_ART.iter().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }
      let t = if ROW_COUNT > 1 {
        i as f32 / (ROW_COUNT - 1) as f32
      } else {
        0.0
      };
      let row_color = lerp_color(start, end, t);
      let row_style = Style::new().fg(row_color);
      let shadow_style = Style::new().fg(shadow_color);

      for ch in row.chars() {
        if SHADOW_CHARS.contains(&ch) {
          write!(f, "{}", ch.paint(shadow_style))?;
        } else {
          write!(f, "{}", ch.paint(row_style))?;
        }
      }
    }

    writeln!(f)?;
    writeln!(
      f,
      "                   {} {}",
      "by @".paint(self.theme.banner_author),
      self.author.paint(self.theme.banner_author_name),
    )?;

    if self.show_version {
      write!(
        f,
        "\n{} {} ({} revision {})",
        format!("v{}", self.version).paint(self.theme.banner_version),
        self.platform.paint(self.theme.banner_version),
        self.date.paint(self.theme.banner_version_date),
        self.revision.paint(self.theme.banner_version_revision),
      )?;
    }

    if let Some(ref update_ver) = self.update_version {
      write!(
        f,
        "\n\n{} {}",
        "a newer version is available:".paint(self.theme.banner_update_message),
        update_ver.paint(self.theme.banner_update_version),
      )?;
      write!(
        f,
        "\n{}{}{}",
        "run ".paint(self.theme.banner_update_hint),
        "gest self-update".paint(self.theme.banner_update_command),
        " to upgrade".paint(self.theme.banner_update_hint),
      )?;
    }

    Ok(())
  }
}

/// Extracts the foreground color from a style, defaulting to white.
fn fg_color(style: &Style) -> Color {
  style.foreground.unwrap_or(Color::White)
}

/// Linearly interpolates between two RGB colors. Falls back to `start` for non-RGB variants.
fn lerp_color(start: Color, end: Color, t: f32) -> Color {
  if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (start, end) {
    Color::Rgb(
      (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8,
      (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8,
      (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8,
    )
  } else {
    start
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn theme() -> Theme {
    Theme::default()
  }

  #[test]
  fn it_lerps_color_at_half() {
    let start = Color::Rgb(0, 0, 0);
    let end = Color::Rgb(100, 200, 50);
    let result = lerp_color(start, end, 0.5);
    assert_eq!(result, Color::Rgb(50, 100, 25));
  }

  #[test]
  fn it_lerps_color_at_one_to_end() {
    let start = Color::Rgb(24, 178, 155);
    let end = Color::Rgb(68, 169, 211);
    let result = lerp_color(start, end, 1.0);
    assert_eq!(result, Color::Rgb(68, 169, 211));
  }

  #[test]
  fn it_lerps_color_at_zero_to_start() {
    let start = Color::Rgb(24, 178, 155);
    let end = Color::Rgb(68, 169, 211);
    let result = lerp_color(start, end, 0.0);
    assert_eq!(result, Color::Rgb(24, 178, 155));
  }

  #[test]
  fn it_omits_update_notice_by_default() {
    let theme = theme();
    let banner = Banner::new("0.2.3", "macos-aarch64", "2026-03-29", "a1b2c3d", "aaronmallen", &theme);
    let rendered = format!("{banner}");

    assert!(!rendered.contains("a newer version is available"));
    assert!(!rendered.contains("self-update"));
  }

  #[test]
  fn it_renders_ascii_art() {
    let theme = theme();
    let banner = Banner::new("0.2.3", "macos-aarch64", "2026-03-29", "a1b2c3d", "aaronmallen", &theme);
    let rendered = format!("{banner}");

    assert!(rendered.contains('█'), "expected block chars in output");
    assert!(rendered.contains('╗'), "expected shadow chars in output");
    assert!(rendered.contains('╚'), "expected shadow chars in output");
    let art_end = rendered.find("by @").expect("should contain author line");
    let art_lines = rendered[..art_end].lines().count();
    assert!(art_lines >= 6, "expected at least 6 ASCII art lines, got {art_lines}");
  }

  #[test]
  fn it_renders_author_line() {
    let theme = theme();
    let banner = Banner::new("0.2.3", "macos-aarch64", "2026-03-29", "a1b2c3d", "aaronmallen", &theme);
    let rendered = format!("{banner}");

    assert!(rendered.contains("by @"));
    assert!(rendered.contains("aaronmallen"));
  }

  #[test]
  fn it_renders_update_notice_when_set() {
    yansi::disable();
    let theme = theme();
    let banner =
      Banner::new("0.2.3", "macos-aarch64", "2026-03-29", "a1b2c3d", "aaronmallen", &theme).update_version("0.2.4");
    let rendered = format!("{banner}");

    assert!(rendered.contains("a newer version is available:"));
    assert!(rendered.contains("0.2.4"));
    assert!(rendered.contains("run gest self-update to upgrade"));
  }

  #[test]
  fn it_renders_version_line() {
    let theme = theme();
    let banner = Banner::new("0.2.3", "macos-aarch64", "2026-03-29", "a1b2c3d", "aaronmallen", &theme);
    let rendered = format!("{banner}");

    assert!(rendered.contains("v0.2.3"), "expected version string, got:\n{rendered}");
    assert!(rendered.contains("macos-aarch64"));
    assert!(rendered.contains("2026-03-29"));
    assert!(rendered.contains("a1b2c3d"));
  }

  #[test]
  fn it_returns_start_for_non_rgb_lerp() {
    let start = Color::Red;
    let end = Color::Blue;
    let result = lerp_color(start, end, 0.5);
    assert_eq!(result, Color::Red);
  }

  #[test]
  fn it_styles_shadow_chars_differently() {
    let has_shadow = ASCII_ART
      .iter()
      .any(|row| row.chars().any(|ch| SHADOW_CHARS.contains(&ch)));
    assert!(has_shadow, "ASCII art should contain shadow characters");
  }
}
