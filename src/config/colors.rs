//! User-configurable color and text style definitions.
//!
//! Colors can be specified as named colors, hex strings (`#RRGGBB`), or
//! tables with foreground, background, and modifier fields.

use std::collections::{HashMap, HashSet};

use getset::Getters;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yansi::{Color, Style};

/// A resolved color and text modifier specification.
#[derive(Clone, Debug, Getters, PartialEq)]
pub struct ColorValue {
  /// Background color, if set.
  #[get = "pub"]
  bg: Option<Color>,
  /// Whether bold weight is enabled.
  #[get = "pub"]
  bold: bool,
  /// Whether dim/faint rendering is enabled.
  #[get = "pub"]
  dim: bool,
  /// Foreground color, if set.
  #[get = "pub"]
  fg: Option<Color>,
  /// Whether italic style is enabled.
  #[get = "pub"]
  italic: bool,
  /// Whether underline decoration is enabled.
  #[get = "pub"]
  underline: bool,
}

impl ColorValue {
  /// Applies this value's colors and modifiers to the given [`Style`].
  pub fn apply_to(&self, mut style: Style) -> Style {
    if let Some(fg) = self.fg {
      style = style.fg(fg);
    }
    if let Some(bg) = self.bg {
      style = style.bg(bg);
    }
    if self.bold {
      style = style.bold();
    }
    if self.dim {
      style = style.dim();
    }
    if self.italic {
      style = style.italic();
    }
    if self.underline {
      style = style.underline();
    }
    style
  }
}

impl<'de> Deserialize<'de> for ColorValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawColorValue {
      String(String),
      Table(RawColorTable),
    }

    #[derive(Deserialize)]
    struct RawColorTable {
      #[serde(default)]
      bg: Option<String>,
      #[serde(default)]
      bold: bool,
      #[serde(default)]
      dim: bool,
      #[serde(default)]
      fg: Option<String>,
      #[serde(default)]
      italic: bool,
      #[serde(default)]
      underline: bool,
    }

    let raw = RawColorValue::deserialize(deserializer)?;
    match raw {
      RawColorValue::String(s) => {
        let color = parse_color(&s).map_err(serde::de::Error::custom)?;
        Ok(ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(color),
          italic: false,
          underline: false,
        })
      }
      RawColorValue::Table(table) => {
        let fg = table
          .fg
          .as_deref()
          .map(parse_color)
          .transpose()
          .map_err(serde::de::Error::custom)?;
        let bg = table
          .bg
          .as_deref()
          .map(parse_color)
          .transpose()
          .map_err(serde::de::Error::custom)?;
        Ok(ColorValue {
          bg,
          bold: table.bold,
          dim: table.dim,
          fg,
          italic: table.italic,
          underline: table.underline,
        })
      }
    }
  }
}

impl Serialize for ColorValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    use serde::ser::SerializeMap;

    if self.bg.is_none()
      && !self.bold
      && !self.dim
      && !self.italic
      && !self.underline
      && let Some(fg) = self.fg
    {
      return serializer.serialize_str(&color_to_string(fg));
    }

    let mut map = serializer.serialize_map(None)?;
    if let Some(fg) = self.fg {
      map.serialize_entry("fg", &color_to_string(fg))?;
    }
    if let Some(bg) = self.bg {
      map.serialize_entry("bg", &color_to_string(bg))?;
    }
    if self.bold {
      map.serialize_entry("bold", &true)?;
    }
    if self.dim {
      map.serialize_entry("dim", &true)?;
    }
    if self.italic {
      map.serialize_entry("italic", &true)?;
    }
    if self.underline {
      map.serialize_entry("underline", &true)?;
    }
    map.end()
  }
}

/// Two-tier color configuration: semantic palette slots and per-token overrides.
///
/// The `palette` map accepts [`Palette`](crate::ui::style::Palette) keys with color-only values.
/// The `overrides` map accepts theme token keys with full [`ColorValue`] format.
#[derive(Clone, Debug, Default, Getters, PartialEq)]
pub struct Settings {
  /// Per-token style overrides keyed by dot-separated token name.
  #[get = "pub"]
  overrides: HashMap<String, ColorValue>,
  /// Palette-level color overrides keyed by slot name (e.g. `"primary"`).
  #[get = "pub"]
  palette: HashMap<String, Color>,
}

impl Settings {
  /// Iterates over all configured token override entries.
  pub fn iter(&self) -> impl Iterator<Item = (&String, &ColorValue)> {
    self.overrides.iter()
  }
}

impl<'de> Deserialize<'de> for Settings {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Default, Deserialize)]
    #[serde(default)]
    struct RawSettings {
      #[serde(default)]
      overrides: HashMap<String, ColorValue>,
      #[serde(default)]
      palette: HashMap<String, String>,
    }

    let raw = RawSettings::deserialize(deserializer)?;

    let valid_keys: HashSet<&str> = crate::ui::style::Palette::ALL.iter().map(|p| p.key()).collect();

    let mut palette = HashMap::new();
    for (key, value) in raw.palette {
      if !valid_keys.contains(key.as_str()) {
        eprintln!("warning: unknown palette key {key:?}");
        continue;
      }
      let color = parse_color(&value).map_err(serde::de::Error::custom)?;
      palette.insert(key, color);
    }

    Ok(Settings {
      overrides: raw.overrides,
      palette,
    })
  }
}

impl Serialize for Settings {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    use serde::ser::SerializeMap;

    let mut map = serializer.serialize_map(None)?;
    if !self.overrides.is_empty() {
      map.serialize_entry("overrides", &self.overrides)?;
    }
    if !self.palette.is_empty() {
      let palette_strings: HashMap<&str, String> = self
        .palette
        .iter()
        .map(|(k, c)| (k.as_str(), color_to_string(*c)))
        .collect();
      map.serialize_entry("palette", &palette_strings)?;
    }
    map.end()
  }
}

/// Converts a [`Color`] back to its canonical string representation.
fn color_to_string(color: Color) -> String {
  match color {
    Color::Black => "black".to_string(),
    Color::Blue => "blue".to_string(),
    Color::BrightBlack => "bright_black".to_string(),
    Color::BrightBlue => "bright_blue".to_string(),
    Color::BrightCyan => "bright_cyan".to_string(),
    Color::BrightGreen => "bright_green".to_string(),
    Color::BrightMagenta => "bright_magenta".to_string(),
    Color::BrightRed => "bright_red".to_string(),
    Color::BrightWhite => "bright_white".to_string(),
    Color::BrightYellow => "bright_yellow".to_string(),
    Color::Cyan => "cyan".to_string(),
    Color::Green => "green".to_string(),
    Color::Magenta => "magenta".to_string(),
    Color::Red => "red".to_string(),
    Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
    Color::White => "white".to_string(),
    Color::Yellow => "yellow".to_string(),
    _ => "white".to_string(),
  }
}

/// Parses a color string, dispatching to hex or named-color parsing.
fn parse_color(s: &str) -> Result<Color, String> {
  if let Some(hex) = s.strip_prefix('#') {
    parse_hex_color(hex)
  } else {
    parse_named_color(s)
  }
}

/// Parses a 6-digit hex color string (without the `#` prefix) into an RGB [`Color`].
fn parse_hex_color(hex: &str) -> Result<Color, String> {
  if hex.len() != 6 {
    return Err(format!("invalid hex color: #{hex} (expected 6 hex digits)"));
  }
  let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| format!("invalid hex color: #{hex}"))?;
  let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| format!("invalid hex color: #{hex}"))?;
  let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| format!("invalid hex color: #{hex}"))?;
  Ok(Color::Rgb(r, g, b))
}

/// Maps a case-insensitive color name (e.g. `"bright_cyan"`) to a [`Color`].
fn parse_named_color(name: &str) -> Result<Color, String> {
  match name.to_lowercase().as_str() {
    "black" => Ok(Color::Black),
    "blue" => Ok(Color::Blue),
    "bright_black" => Ok(Color::BrightBlack),
    "bright_blue" => Ok(Color::BrightBlue),
    "bright_cyan" => Ok(Color::BrightCyan),
    "bright_green" => Ok(Color::BrightGreen),
    "bright_magenta" => Ok(Color::BrightMagenta),
    "bright_red" => Ok(Color::BrightRed),
    "bright_white" => Ok(Color::BrightWhite),
    "bright_yellow" => Ok(Color::BrightYellow),
    "cyan" => Ok(Color::Cyan),
    "green" => Ok(Color::Green),
    "magenta" => Ok(Color::Magenta),
    "red" => Ok(Color::Red),
    "white" => Ok(Color::White),
    "yellow" => Ok(Color::Yellow),
    _ => Err(format!("unknown color name: {name}")),
  }
}

#[cfg(test)]
mod tests {
  use toml::Value;
  use yansi::{Color, Style};

  use super::*;

  mod color_to_string {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_named_colors() {
      assert_eq!(color_to_string(Color::Black), "black");
      assert_eq!(color_to_string(Color::Blue), "blue");
      assert_eq!(color_to_string(Color::BrightBlack), "bright_black");
      assert_eq!(color_to_string(Color::BrightBlue), "bright_blue");
      assert_eq!(color_to_string(Color::BrightCyan), "bright_cyan");
      assert_eq!(color_to_string(Color::BrightGreen), "bright_green");
      assert_eq!(color_to_string(Color::BrightMagenta), "bright_magenta");
      assert_eq!(color_to_string(Color::BrightRed), "bright_red");
      assert_eq!(color_to_string(Color::BrightWhite), "bright_white");
      assert_eq!(color_to_string(Color::BrightYellow), "bright_yellow");
      assert_eq!(color_to_string(Color::Cyan), "cyan");
      assert_eq!(color_to_string(Color::Green), "green");
      assert_eq!(color_to_string(Color::Magenta), "magenta");
      assert_eq!(color_to_string(Color::Red), "red");
      assert_eq!(color_to_string(Color::White), "white");
      assert_eq!(color_to_string(Color::Yellow), "yellow");
    }

    #[test]
    fn it_converts_rgb_to_uppercase_hex() {
      assert_eq!(color_to_string(Color::Rgb(255, 128, 0)), "#FF8000");
      assert_eq!(color_to_string(Color::Rgb(0, 0, 0)), "#000000");
      assert_eq!(color_to_string(Color::Rgb(255, 255, 255)), "#FFFFFF");
    }

    #[test]
    fn it_falls_back_to_white_for_unknown_variants() {
      assert_eq!(color_to_string(Color::Fixed(42)), "white");
    }
  }

  mod color_value_apply_to {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_foreground_color() {
      let value = ColorValue {
        bg: None,
        bold: false,
        dim: false,
        fg: Some(Color::Red),
        italic: false,
        underline: false,
      };

      let style = value.apply_to(Style::new());

      assert_eq!(style, Style::new().fg(Color::Red));
    }

    #[test]
    fn it_applies_background_color() {
      let value = ColorValue {
        bg: Some(Color::Blue),
        bold: false,
        dim: false,
        fg: None,
        italic: false,
        underline: false,
      };

      let style = value.apply_to(Style::new());

      assert_eq!(style, Style::new().bg(Color::Blue));
    }

    #[test]
    fn it_applies_all_modifiers() {
      let value = ColorValue {
        bg: None,
        bold: true,
        dim: true,
        fg: None,
        italic: true,
        underline: true,
      };

      let style = value.apply_to(Style::new());

      assert_eq!(style, Style::new().bold().dim().italic().underline());
    }

    #[test]
    fn it_applies_everything_together() {
      let value = ColorValue {
        bg: Some(Color::Black),
        bold: true,
        dim: false,
        fg: Some(Color::White),
        italic: true,
        underline: false,
      };

      let style = value.apply_to(Style::new());

      assert_eq!(style, Style::new().fg(Color::White).bg(Color::Black).bold().italic());
    }

    #[test]
    fn it_preserves_existing_style_properties() {
      let value = ColorValue {
        bg: None,
        bold: true,
        dim: false,
        fg: None,
        italic: false,
        underline: false,
      };

      let style = value.apply_to(Style::new().fg(Color::Green));

      assert_eq!(style, Style::new().fg(Color::Green).bold());
    }
  }

  mod color_value_deserialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deserializes_a_string_as_fg_only() {
      let value: ColorValue = toml::from_str::<Value>("color = \"red\"")
        .unwrap()
        .get("color")
        .unwrap()
        .clone()
        .try_into()
        .unwrap();

      assert_eq!(value.fg, Some(Color::Red));
      assert_eq!(value.bg, None);
      assert!(!value.bold);
      assert!(!value.dim);
      assert!(!value.italic);
      assert!(!value.underline);
    }

    #[test]
    fn it_deserializes_a_hex_string() {
      let value: ColorValue = toml::from_str::<Value>("color = \"#FF8000\"")
        .unwrap()
        .get("color")
        .unwrap()
        .clone()
        .try_into()
        .unwrap();

      assert_eq!(value.fg, Some(Color::Rgb(255, 128, 0)));
    }

    #[test]
    fn it_deserializes_a_full_table() {
      let toml_str = r#"
        [color]
        fg = "green"
        bg = "black"
        bold = true
        italic = true
      "#;

      let value: ColorValue = toml::from_str::<Value>(toml_str)
        .unwrap()
        .get("color")
        .unwrap()
        .clone()
        .try_into()
        .unwrap();

      assert_eq!(value.fg, Some(Color::Green));
      assert_eq!(value.bg, Some(Color::Black));
      assert!(value.bold);
      assert!(!value.dim);
      assert!(value.italic);
      assert!(!value.underline);
    }

    #[test]
    fn it_defaults_missing_table_fields() {
      let toml_str = r#"
        [color]
        fg = "cyan"
      "#;

      let value: ColorValue = toml::from_str::<Value>(toml_str)
        .unwrap()
        .get("color")
        .unwrap()
        .clone()
        .try_into()
        .unwrap();

      assert_eq!(value.fg, Some(Color::Cyan));
      assert_eq!(value.bg, None);
      assert!(!value.bold);
      assert!(!value.dim);
      assert!(!value.italic);
      assert!(!value.underline);
    }

    #[test]
    fn it_rejects_invalid_color_names() {
      let result: Result<ColorValue, _> = toml::from_str::<Value>("color = \"nope\"")
        .unwrap()
        .get("color")
        .unwrap()
        .clone()
        .try_into();

      assert!(result.is_err());
    }
  }

  mod color_value_serialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Wrapper {
      color: ColorValue,
    }

    #[test]
    fn it_serializes_fg_only_as_string() {
      let wrapper = Wrapper {
        color: ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(Color::Red),
          italic: false,
          underline: false,
        },
      };

      let serialized = toml::to_string(&wrapper).unwrap();

      assert!(
        serialized.contains("\"red\""),
        "expected string form, got: {serialized}"
      );
    }

    #[test]
    fn it_serializes_as_table_when_modifiers_present() {
      let wrapper = Wrapper {
        color: ColorValue {
          bg: None,
          bold: true,
          dim: false,
          fg: Some(Color::Green),
          italic: false,
          underline: false,
        },
      };

      let serialized = toml::to_string(&wrapper).unwrap();

      assert!(serialized.contains("fg"), "expected table form, got: {serialized}");
      assert!(serialized.contains("bold"), "expected bold key, got: {serialized}");
    }

    #[test]
    fn it_serializes_as_table_when_bg_present() {
      let wrapper = Wrapper {
        color: ColorValue {
          bg: Some(Color::Black),
          bold: false,
          dim: false,
          fg: Some(Color::White),
          italic: false,
          underline: false,
        },
      };

      let serialized = toml::to_string(&wrapper).unwrap();

      assert!(serialized.contains("bg"), "expected bg key, got: {serialized}");
    }

    #[test]
    fn it_roundtrips_through_serialize_deserialize() {
      let original = ColorValue {
        bg: Some(Color::Rgb(10, 20, 30)),
        bold: true,
        dim: false,
        fg: Some(Color::Rgb(255, 128, 0)),
        italic: true,
        underline: false,
      };

      #[derive(Serialize, Deserialize)]
      struct Wrapper {
        color: ColorValue,
      }

      let serialized = toml::to_string(&Wrapper {
        color: original.clone(),
      })
      .unwrap();
      let deserialized: Wrapper = toml::from_str(&serialized).unwrap();

      assert_eq!(deserialized.color, original);
    }
  }

  mod parse_color {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_dispatches_hex_strings_to_hex_parser() {
      let result = parse_color("#FF0000").unwrap();

      assert_eq!(result, Color::Rgb(255, 0, 0));
    }

    #[test]
    fn it_dispatches_bare_names_to_named_parser() {
      let result = parse_color("red").unwrap();

      assert_eq!(result, Color::Red);
    }
  }

  mod parse_hex_color {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_a_valid_hex_string() {
      let result = parse_hex_color("4EA8E0").unwrap();

      assert_eq!(result, Color::Rgb(78, 168, 224));
    }

    #[test]
    fn it_parses_lowercase_hex() {
      let result = parse_hex_color("ff8000").unwrap();

      assert_eq!(result, Color::Rgb(255, 128, 0));
    }

    #[test]
    fn it_rejects_too_short_hex() {
      let result = parse_hex_color("FFF");

      assert!(result.is_err());
      assert!(result.unwrap_err().contains("expected 6 hex digits"));
    }

    #[test]
    fn it_rejects_too_long_hex() {
      let result = parse_hex_color("FF00FF00");

      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_invalid_hex_characters() {
      let result = parse_hex_color("GGHHII");

      assert!(result.is_err());
      assert!(result.unwrap_err().contains("invalid hex color"));
    }
  }

  mod parse_named_color {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_all_standard_colors() {
      let cases = [
        ("black", Color::Black),
        ("blue", Color::Blue),
        ("cyan", Color::Cyan),
        ("green", Color::Green),
        ("magenta", Color::Magenta),
        ("red", Color::Red),
        ("white", Color::White),
        ("yellow", Color::Yellow),
      ];

      for (name, expected) in cases {
        assert_eq!(parse_named_color(name).unwrap(), expected, "failed for {name}");
      }
    }

    #[test]
    fn it_parses_all_bright_colors() {
      let cases = [
        ("bright_black", Color::BrightBlack),
        ("bright_blue", Color::BrightBlue),
        ("bright_cyan", Color::BrightCyan),
        ("bright_green", Color::BrightGreen),
        ("bright_magenta", Color::BrightMagenta),
        ("bright_red", Color::BrightRed),
        ("bright_white", Color::BrightWhite),
        ("bright_yellow", Color::BrightYellow),
      ];

      for (name, expected) in cases {
        assert_eq!(parse_named_color(name).unwrap(), expected, "failed for {name}");
      }
    }

    #[test]
    fn it_is_case_insensitive() {
      assert_eq!(parse_named_color("RED").unwrap(), Color::Red);
      assert_eq!(parse_named_color("Bright_Cyan").unwrap(), Color::BrightCyan);
    }

    #[test]
    fn it_rejects_unknown_names() {
      let result = parse_named_color("chartreuse");

      assert!(result.is_err());
      assert!(result.unwrap_err().contains("unknown color name"));
    }
  }

  mod settings_deserialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deserializes_empty_table() {
      let settings: Settings = toml::from_str("").unwrap();

      assert!(settings.palette.is_empty());
      assert!(settings.overrides.is_empty());
    }

    #[test]
    fn it_deserializes_palette_entries() {
      let toml_str = r##"
        [palette]
        primary = "blue"
        error = "#D03838"
      "##;

      let settings: Settings = toml::from_str(toml_str).unwrap();

      assert_eq!(*settings.palette.get("primary").unwrap(), Color::Blue);
      assert_eq!(*settings.palette.get("error").unwrap(), Color::Rgb(208, 56, 56));
    }

    #[test]
    fn it_warns_on_unknown_palette_keys() {
      let toml_str = r#"
        [palette]
        primary = "blue"
        nonexistent = "red"
      "#;

      let settings: Settings = toml::from_str(toml_str).unwrap();

      assert!(settings.palette.contains_key("primary"));
      assert!(!settings.palette.contains_key("nonexistent"));
    }

    #[test]
    fn it_deserializes_overrides() {
      let toml_str = r#"
        [overrides.some_token]
        fg = "yellow"
        bold = true
      "#;

      let settings: Settings = toml::from_str(toml_str).unwrap();
      let token = settings.overrides.get("some_token").unwrap();

      assert_eq!(token.fg, Some(Color::Yellow));
      assert!(token.bold);
    }

    #[test]
    fn it_rejects_invalid_palette_colors() {
      let toml_str = r#"
        [palette]
        primary = "invalid_color"
      "#;

      let result: Result<Settings, _> = toml::from_str(toml_str);

      assert!(result.is_err());
    }
  }

  mod settings_iter {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_iterates_over_overrides() {
      let mut overrides = HashMap::new();
      overrides.insert(
        "a".to_string(),
        ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(Color::Red),
          italic: false,
          underline: false,
        },
      );

      let settings = Settings {
        overrides,
        palette: HashMap::new(),
      };
      let entries: Vec<_> = settings.iter().collect();

      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].0, "a");
    }
  }

  mod settings_serialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_omits_empty_sections() {
      let settings = Settings::default();

      let serialized = toml::to_string(&settings).unwrap();

      assert!(serialized.trim().is_empty());
    }

    #[test]
    fn it_serializes_palette_as_string_values() {
      let mut palette = HashMap::new();
      palette.insert("primary".to_string(), Color::Blue);

      let settings = Settings {
        overrides: HashMap::new(),
        palette,
      };

      let serialized = toml::to_string(&settings).unwrap();

      assert!(
        serialized.contains("primary"),
        "expected palette key, got: {serialized}"
      );
      assert!(serialized.contains("blue"), "expected color string, got: {serialized}");
    }

    #[test]
    fn it_roundtrips_through_serialize_deserialize() {
      let mut palette = HashMap::new();
      palette.insert("primary".to_string(), Color::Rgb(100, 150, 200));

      let mut overrides = HashMap::new();
      overrides.insert(
        "test.token".to_string(),
        ColorValue {
          bg: None,
          bold: true,
          dim: false,
          fg: Some(Color::Green),
          italic: false,
          underline: false,
        },
      );

      let original = Settings {
        overrides,
        palette,
      };

      let serialized = toml::to_string(&original).unwrap();
      let deserialized: Settings = toml::from_str(&serialized).unwrap();

      assert_eq!(deserialized, original);
    }
  }
}
