//! Startup banner molecule with gradient ASCII art and optional author/version lines.

use std::fmt::{self, Display, Formatter};

use yansi::{Color, Paint, Style};

/// ASCII block-art rows for the GEST logo.
const ASCII_ART: [&str; 6] = [
  " ██████╗ ███████╗███████╗████████╗",
  "██╔════╝ ██╔════╝██╔════╝╚══██╔══╝",
  "██║  ███╗█████╗  ███████╗   ██║",
  "██║   ██║██╔══╝  ╚════██║   ██║",
  "╚██████╔╝███████╗███████║   ██║",
  " ╚═════╝ ╚══════╝╚══════╝   ╚═╝",
];

/// Total number of rows in the ASCII art, used to compute gradient interpolation.
const ROW_COUNT: usize = ASCII_ART.len();

/// Box-drawing characters rendered with the shadow color for depth effect.
const SHADOW_CHARS: &[char] = &['╗', '╚', '╝', '╔', '║'];

/// Renders the application startup banner with gradient ASCII art and optional
/// author and version lines.
pub struct Component {
  show_author: bool,
  show_version: bool,
}

impl Component {
  /// Create a banner with no optional lines enabled.
  pub fn new() -> Self {
    Self {
      show_author: false,
      show_version: false,
    }
  }

  /// Enable the author attribution line beneath the ASCII art.
  pub fn with_author(mut self) -> Self {
    self.show_author = true;
    self
  }

  /// Enable the version info line beneath the ASCII art.
  pub fn with_version(mut self) -> Self {
    self.show_version = true;
    self
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let theme = crate::ui::style::global();
    let start = fg_color(theme.banner_gradient_start());
    let end = fg_color(theme.banner_gradient_end());
    let shadow_color = fg_color(theme.banner_shadow());

    for (i, row) in ASCII_ART.iter().enumerate() {
      if i > 0 {
        writeln!(f)?;
      }
      let t = if ROW_COUNT > 1 {
        i as f32 / (ROW_COUNT - 1) as f32
      } else {
        0.0
      };
      let row_style = Style::new().fg(lerp_color(start, end, t));
      let shadow_style = Style::new().fg(shadow_color);

      for ch in row.chars() {
        if SHADOW_CHARS.contains(&ch) {
          write!(f, "{}", ch.paint(shadow_style))?;
        } else {
          write!(f, "{}", ch.paint(row_style))?;
        }
      }
    }

    if self.show_author {
      writeln!(f)?;
      write!(
        f,
        "                   by @{}",
        "aaronmallen".paint(*theme.banner_author())
      )?;
    }

    if self.show_version {
      writeln!(f)?;
      write!(
        f,
        "\nv{} {}-{} ({} revision {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("BUILD_DATE").paint(*theme.banner_version_date()),
        env!("GIT_SHA").paint(*theme.banner_version_revision()),
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

  mod fg_color_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_defaults_to_white_when_no_foreground() {
      let style = Style::new();

      assert_eq!(fg_color(&style), Color::White);
    }

    #[test]
    fn it_extracts_the_foreground_color() {
      let style = Style::new().fg(Color::Rgb(10, 20, 30));

      assert_eq!(fg_color(&style), Color::Rgb(10, 20, 30));
    }
  }

  mod lerp_color_fn {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_falls_back_to_start_for_non_rgb_colors() {
      let start = Color::Red;
      let end = Color::Blue;

      assert_eq!(lerp_color(start, end, 0.5), Color::Red);
    }

    #[test]
    fn it_falls_back_when_only_end_is_non_rgb() {
      let start = Color::Rgb(100, 100, 100);
      let end = Color::Blue;

      assert_eq!(lerp_color(start, end, 0.5), Color::Rgb(100, 100, 100));
    }

    #[test]
    fn it_falls_back_when_only_start_is_non_rgb() {
      let start = Color::Red;
      let end = Color::Rgb(100, 100, 100);

      assert_eq!(lerp_color(start, end, 0.5), Color::Red);
    }

    #[test]
    fn it_interpolates_at_midpoint() {
      let start = Color::Rgb(0, 0, 0);
      let end = Color::Rgb(100, 200, 50);

      assert_eq!(lerp_color(start, end, 0.5), Color::Rgb(50, 100, 25));
    }

    #[test]
    fn it_returns_end_at_t_one() {
      let start = Color::Rgb(0, 100, 200);
      let end = Color::Rgb(200, 50, 0);

      assert_eq!(lerp_color(start, end, 1.0), end);
    }

    #[test]
    fn it_returns_start_at_t_zero() {
      let start = Color::Rgb(0, 100, 200);
      let end = Color::Rgb(200, 50, 0);

      assert_eq!(lerp_color(start, end, 0.0), start);
    }
  }

  mod fmt {
    use super::*;

    #[test]
    fn it_does_not_include_author_by_default() {
      let banner = Component::new();

      let output = banner.to_string();

      assert!(!output.contains("aaronmallen"), "author should not appear by default");
    }

    #[test]
    fn it_does_not_include_version_by_default() {
      let banner = Component::new();

      let output = banner.to_string();

      assert!(
        !output.contains(env!("CARGO_PKG_VERSION")),
        "version should not appear by default"
      );
    }

    #[test]
    fn it_includes_author_when_enabled() {
      let banner = Component::new().with_author();

      let output = banner.to_string();

      assert!(output.contains("aaronmallen"), "output should contain author");
    }

    #[test]
    fn it_includes_both_author_and_version_when_enabled() {
      let banner = Component::new().with_author().with_version();

      let output = banner.to_string();

      assert!(output.contains("aaronmallen"), "output should contain author");
      assert!(
        output.contains(env!("CARGO_PKG_VERSION")),
        "output should contain version"
      );
    }

    #[test]
    fn it_includes_version_when_enabled() {
      let banner = Component::new().with_version();

      let output = banner.to_string();

      assert!(
        output.contains(env!("CARGO_PKG_VERSION")),
        "output should contain the version"
      );
    }

    #[test]
    fn it_renders_all_ascii_art_rows() {
      let banner = Component::new();

      let output = banner.to_string();
      let lines: Vec<&str> = output.lines().collect();

      assert_eq!(lines.len(), ROW_COUNT, "banner should have {ROW_COUNT} lines");
    }
  }
}
