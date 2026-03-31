//! Named RGB color constants that form the application's visual palette.

use yansi::Color;

/// Warm yellow accent for warnings and in-progress states.
pub const AMBER: Color = Color::Rgb(204, 152, 32);

/// Primary brand blue.
pub const AZURE: Color = Color::Rgb(78, 168, 224);

/// Darker shade of azure for subtle accents.
pub const AZURE_DARK: Color = Color::Rgb(50, 120, 176);

/// Lighter shade of azure for highlights.
pub const AZURE_LIGHT: Color = Color::Rgb(124, 196, 240);

/// Subtle rule / border gray.
pub const BORDER: Color = Color::Rgb(48, 50, 58);

/// De-emphasized foreground.
pub const DIM: Color = Color::Rgb(88, 94, 110);

/// Warm orange accent.
pub const EMBER: Color = Color::Rgb(208, 88, 48);

/// Semantic error red.
pub const ERROR: Color = Color::Rgb(208, 56, 56);

/// Semantic success green.
pub const JADE: Color = Color::Rgb(54, 190, 120);

/// Mid-tone neutral for secondary text.
pub const PEWTER: Color = Color::Rgb(124, 130, 148);

/// Light neutral for primary content.
pub const SILVER: Color = Color::Rgb(196, 200, 212);

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_rgb(color: Color, r: u8, g: u8, b: u8) {
    assert_eq!(color, Color::Rgb(r, g, b));
  }

  #[test]
  fn it_verifies_accent_colors() {
    assert_rgb(AMBER, 204, 152, 32);
    assert_rgb(JADE, 54, 190, 120);
    assert_rgb(EMBER, 208, 88, 48);
  }

  #[test]
  fn it_verifies_brand_colors() {
    assert_rgb(AZURE, 78, 168, 224);
    assert_rgb(AZURE_DARK, 50, 120, 176);
    assert_rgb(AZURE_LIGHT, 124, 196, 240);
  }

  #[test]
  fn it_verifies_core_neutrals() {
    assert_rgb(BORDER, 48, 50, 58);
    assert_rgb(DIM, 88, 94, 110);
    assert_rgb(PEWTER, 124, 130, 148);
    assert_rgb(SILVER, 196, 200, 212);
  }

  #[test]
  fn it_verifies_semantic_colors() {
    assert_rgb(ERROR, 208, 56, 56);
  }
}
