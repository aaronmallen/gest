use yansi::Style;

use super::colors;
use crate::config::Config;

/// Semantic color theme for CLI output.
///
/// Each field holds a [`yansi::Style`] for a specific semantic token.
/// The [`Default`] implementation uses the brand palette RGB values.
#[derive(Clone, Debug)]
pub struct Theme {
  pub border: Style,
  pub emphasis: Style,
  pub error: Style,
  pub id_prefix: Style,
  pub id_rest: Style,
  pub indicator_blocked: Style,
  pub indicator_blocking: Style,
  pub list_heading: Style,
  pub log_debug: Style,
  pub log_error: Style,
  pub log_info: Style,
  pub log_trace: Style,
  pub log_warn: Style,
  pub md_blockquote: Style,
  pub md_bold: Style,
  pub md_code: Style,
  pub md_code_block: Style,
  pub md_heading: Style,
  pub md_italic: Style,
  pub md_link: Style,
  pub md_rule: Style,
  pub muted: Style,
  pub status_cancelled: Style,
  pub status_done: Style,
  pub status_in_progress: Style,
  pub status_open: Style,
  pub success: Style,
  pub tag: Style,
}

impl Theme {
  /// Construct a theme by merging user overrides from config with defaults.
  ///
  /// For each entry in `config.colors`, the dot-separated key (e.g. `"log.error"`)
  /// is mapped to the corresponding theme field, and the [`ColorValue`] is applied
  /// on top of the default style. Unknown keys are silently ignored.
  pub fn from_config(config: &Config) -> Self {
    let mut theme = Self::default();

    for (key, value) in &config.colors {
      match key.as_str() {
        "border" => theme.border = value.apply_to(theme.border),
        "emphasis" => theme.emphasis = value.apply_to(theme.emphasis),
        "error" => theme.error = value.apply_to(theme.error),
        "id_prefix" => theme.id_prefix = value.apply_to(theme.id_prefix),
        "id_rest" => theme.id_rest = value.apply_to(theme.id_rest),
        "indicator.blocked" => theme.indicator_blocked = value.apply_to(theme.indicator_blocked),
        "indicator.blocking" => theme.indicator_blocking = value.apply_to(theme.indicator_blocking),
        "list_heading" => theme.list_heading = value.apply_to(theme.list_heading),
        "log.debug" => theme.log_debug = value.apply_to(theme.log_debug),
        "log.error" => theme.log_error = value.apply_to(theme.log_error),
        "log.info" => theme.log_info = value.apply_to(theme.log_info),
        "log.trace" => theme.log_trace = value.apply_to(theme.log_trace),
        "log.warn" => theme.log_warn = value.apply_to(theme.log_warn),
        "md.blockquote" => theme.md_blockquote = value.apply_to(theme.md_blockquote),
        "md.bold" => theme.md_bold = value.apply_to(theme.md_bold),
        "md.code" => theme.md_code = value.apply_to(theme.md_code),
        "md.code_block" => theme.md_code_block = value.apply_to(theme.md_code_block),
        "md.heading" => theme.md_heading = value.apply_to(theme.md_heading),
        "md.italic" => theme.md_italic = value.apply_to(theme.md_italic),
        "md.link" => theme.md_link = value.apply_to(theme.md_link),
        "md.rule" => theme.md_rule = value.apply_to(theme.md_rule),
        "muted" => theme.muted = value.apply_to(theme.muted),
        "status.cancelled" => theme.status_cancelled = value.apply_to(theme.status_cancelled),
        "status.done" => theme.status_done = value.apply_to(theme.status_done),
        "status.in_progress" => theme.status_in_progress = value.apply_to(theme.status_in_progress),
        "status.open" => theme.status_open = value.apply_to(theme.status_open),
        "success" => theme.success = value.apply_to(theme.success),
        "tag" => theme.tag = value.apply_to(theme.tag),
        _ => log::warn!("unknown color token: {key}"),
      }
    }

    theme
  }
}

impl Default for Theme {
  fn default() -> Self {
    Self {
      border: Style::new().fg(colors::BORDER),
      emphasis: Style::new().fg(colors::VIOLET).bold(),
      error: Style::new().fg(colors::ERROR).bold(),
      id_prefix: Style::new().fg(colors::AZURE).bold(),
      id_rest: Style::new().fg(colors::PEWTER),
      indicator_blocked: Style::new().fg(colors::ERROR).bold(),
      indicator_blocking: Style::new().fg(colors::WARNING).bold(),
      list_heading: Style::new().fg(colors::VIOLET).bold().underline(),
      log_debug: Style::new().fg(colors::VIOLET_LIGHT),
      log_error: Style::new().fg(colors::ERROR),
      log_info: Style::new().fg(colors::AZURE),
      log_trace: Style::new().fg(colors::DIM),
      log_warn: Style::new().fg(colors::WARNING),
      md_blockquote: Style::new().fg(colors::PEWTER).italic(),
      md_bold: Style::new().bold(),
      md_code: Style::new().fg(colors::EMBER),
      md_code_block: Style::new().fg(colors::SILVER),
      md_heading: Style::new().fg(colors::VIOLET).bold(),
      md_italic: Style::new().italic(),
      md_link: Style::new().fg(colors::AZURE).underline(),
      md_rule: Style::new().fg(colors::BORDER),
      muted: Style::new().fg(colors::PEWTER),
      status_cancelled: Style::new().fg(colors::DIM),
      status_done: Style::new().fg(colors::SUCCESS),
      status_in_progress: Style::new().fg(colors::WARNING),
      status_open: Style::new().fg(colors::SILVER),
      success: Style::new().fg(colors::SUCCESS).bold(),
      tag: Style::new().fg(colors::AZURE).italic(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::ColorValue;

  mod from_config {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use yansi::Color;

    use super::*;

    #[test]
    fn it_ignores_unknown_token_names() {
      let mut config = Config::default();
      config.colors.insert(
        "nonexistent.token".to_string(),
        ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(Color::Red),
          italic: false,
          underline: false,
        },
      );

      // Should not panic
      let theme = Theme::from_config(&config);

      assert_eq!(theme.log_error, Theme::default().log_error);
    }

    #[test]
    fn it_maps_all_token_names() {
      let red = ColorValue {
        bg: None,
        bold: false,
        dim: false,
        fg: Some(Color::Red),
        italic: false,
        underline: false,
      };

      let mut colors = HashMap::new();
      colors.insert("log.debug".to_string(), red.clone());
      colors.insert("log.error".to_string(), red.clone());
      colors.insert("log.info".to_string(), red.clone());
      colors.insert("log.trace".to_string(), red.clone());
      colors.insert("log.warn".to_string(), red.clone());
      colors.insert("md.blockquote".to_string(), red.clone());
      colors.insert("md.bold".to_string(), red.clone());
      colors.insert("md.code".to_string(), red.clone());
      colors.insert("md.code_block".to_string(), red.clone());
      colors.insert("md.heading".to_string(), red.clone());
      colors.insert("md.italic".to_string(), red.clone());
      colors.insert("md.link".to_string(), red.clone());
      colors.insert("md.rule".to_string(), red.clone());
      colors.insert("status.cancelled".to_string(), red.clone());
      colors.insert("status.done".to_string(), red.clone());
      colors.insert("status.in_progress".to_string(), red.clone());
      colors.insert("status.open".to_string(), red.clone());
      colors.insert("border".to_string(), red.clone());
      colors.insert("emphasis".to_string(), red.clone());
      colors.insert("error".to_string(), red.clone());
      colors.insert("id_prefix".to_string(), red.clone());
      colors.insert("id_rest".to_string(), red.clone());
      colors.insert("indicator.blocked".to_string(), red.clone());
      colors.insert("indicator.blocking".to_string(), red.clone());
      colors.insert("list_heading".to_string(), red.clone());
      colors.insert("muted".to_string(), red.clone());
      colors.insert("success".to_string(), red.clone());
      colors.insert("tag".to_string(), red.clone());

      let mut config = Config::default();
      config.colors = colors;

      let theme = Theme::from_config(&config);
      let red = Style::new().fg(Color::Red);
      let red_bold = Style::new().fg(Color::Red).bold();
      let red_bold_underline = Style::new().fg(Color::Red).bold().underline();
      let red_italic = Style::new().fg(Color::Red).italic();
      let red_underline = Style::new().fg(Color::Red).underline();

      // Tokens without default modifiers get plain red
      assert_eq!(theme.log_debug, red);
      assert_eq!(theme.log_error, red);
      assert_eq!(theme.log_info, red);
      assert_eq!(theme.log_trace, red);
      assert_eq!(theme.log_warn, red);
      assert_eq!(theme.md_code, red);
      assert_eq!(theme.md_code_block, red);
      assert_eq!(theme.md_rule, red);
      assert_eq!(theme.status_cancelled, red);
      assert_eq!(theme.status_done, red);
      assert_eq!(theme.status_in_progress, red);
      assert_eq!(theme.status_open, red);
      assert_eq!(theme.border, red);
      assert_eq!(theme.id_rest, red);
      assert_eq!(theme.muted, red);
      // Tokens with default italic keep it
      assert_eq!(theme.tag, red_italic);
      // Tokens with default bold keep it (apply_to layers on top)
      assert_eq!(theme.emphasis, red_bold);
      assert_eq!(theme.error, red_bold);
      assert_eq!(theme.id_prefix, red_bold);
      assert_eq!(theme.indicator_blocked, red_bold);
      assert_eq!(theme.indicator_blocking, red_bold);
      assert_eq!(theme.md_bold, red_bold);
      assert_eq!(theme.md_heading, red_bold);
      assert_eq!(theme.success, red_bold);
      // Tokens with default bold+underline keep both
      assert_eq!(theme.list_heading, red_bold_underline);
      // Tokens with default italic keep it
      assert_eq!(theme.md_blockquote, red_italic);
      assert_eq!(theme.md_italic, red_italic);
      // Tokens with default underline keep it
      assert_eq!(theme.md_link, red_underline);
    }

    #[test]
    fn it_overrides_a_single_token() {
      let mut config = Config::default();
      config.colors.insert(
        "log.error".to_string(),
        ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(Color::Rgb(255, 0, 0)),
          italic: false,
          underline: false,
        },
      );

      let theme = Theme::from_config(&config);

      assert_eq!(theme.log_error, Style::new().fg(Color::Rgb(255, 0, 0)));
      // Other tokens remain at defaults
      assert_eq!(theme.log_warn, Theme::default().log_warn);
    }

    #[test]
    fn it_overrides_multiple_tokens() {
      let mut config = Config::default();
      config.colors.insert(
        "log.error".to_string(),
        ColorValue {
          bg: None,
          bold: false,
          dim: false,
          fg: Some(Color::Red),
          italic: false,
          underline: false,
        },
      );
      config.colors.insert(
        "emphasis".to_string(),
        ColorValue {
          bg: None,
          bold: true,
          dim: false,
          fg: Some(Color::Rgb(148, 72, 199)),
          italic: false,
          underline: false,
        },
      );

      let theme = Theme::from_config(&config);

      assert_eq!(theme.log_error, Style::new().fg(Color::Red));
      assert_eq!(theme.emphasis, Style::new().fg(Color::Rgb(148, 72, 199)).bold());
    }

    #[test]
    fn it_returns_defaults_with_empty_colors() {
      let config = Config::default();

      let theme = Theme::from_config(&config);

      assert_eq!(theme.log_error, Theme::default().log_error);
      assert_eq!(theme.emphasis, Theme::default().emphasis);
    }
  }
}
