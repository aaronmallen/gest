use yansi::Style;

use super::colors;

/// Semantic style tokens for all UI elements.
///
/// Each field maps a named UI role to a [`yansi::Style`].  The [`Default`]
/// implementation uses the brand palette; user overrides from config are
/// applied via [`Theme::apply_overrides`].
#[derive(Debug, Clone)]
pub struct Theme {
  pub artifact_detail_label: Style,
  pub artifact_detail_separator: Style,
  pub artifact_detail_value: Style,
  pub artifact_list_archived_badge: Style,
  pub artifact_list_kind: Style,
  pub artifact_list_tag_archived: Style,
  pub artifact_list_title: Style,
  pub artifact_list_title_archived: Style,

  pub banner_author: Style,
  pub banner_author_name: Style,
  pub banner_gradient_end: Style,
  pub banner_gradient_start: Style,
  pub banner_shadow: Style,
  pub banner_update_command: Style,
  pub banner_update_hint: Style,
  pub banner_update_message: Style,
  pub banner_update_version: Style,
  pub banner_version: Style,
  pub banner_version_date: Style,
  pub banner_version_revision: Style,

  pub border: Style,

  pub config_heading: Style,
  pub config_label: Style,
  pub config_no_overrides: Style,
  pub config_value: Style,

  pub emphasis: Style,
  pub error: Style,

  pub id_prefix: Style,
  pub id_rest: Style,

  pub indicator_blocked: Style,
  pub indicator_blocked_by_id: Style,
  pub indicator_blocked_by_label: Style,
  pub indicator_blocking: Style,

  pub init_command_prefix: Style,
  pub init_label: Style,
  pub init_section: Style,
  pub init_value: Style,

  pub iteration_detail_count_blocked: Style,
  pub iteration_detail_count_done: Style,
  pub iteration_detail_count_in_progress: Style,
  pub iteration_detail_count_open: Style,
  pub iteration_detail_label: Style,
  pub iteration_detail_value: Style,

  pub iteration_graph_branch: Style,
  pub iteration_graph_phase_icon: Style,
  pub iteration_graph_phase_label: Style,
  pub iteration_graph_phase_name: Style,
  pub iteration_graph_separator: Style,
  pub iteration_graph_title: Style,

  pub iteration_list_summary: Style,
  pub iteration_list_title: Style,

  pub list_heading: Style,
  pub list_summary: Style,

  pub log_debug: Style,
  pub log_error: Style,
  pub log_info: Style,
  pub log_timestamp: Style,
  pub log_trace: Style,
  pub log_warn: Style,

  pub markdown_blockquote: Style,
  pub markdown_blockquote_border: Style,
  pub markdown_code_block: Style,
  pub markdown_code_border: Style,
  pub markdown_code_inline: Style,
  pub markdown_emphasis: Style,
  pub markdown_heading: Style,
  pub markdown_link: Style,
  pub markdown_rule: Style,
  pub markdown_strong: Style,

  pub message_created_label: Style,
  pub message_success_icon: Style,
  pub message_updated_label: Style,

  pub muted: Style,

  pub search_expand_separator: Style,
  pub search_no_results_hint: Style,
  pub search_query: Style,
  pub search_summary: Style,
  pub search_type_label: Style,

  pub status_cancelled: Style,
  pub status_done: Style,
  pub status_in_progress: Style,
  pub status_open: Style,

  pub success: Style,
  pub tag: Style,

  pub task_detail_label: Style,
  pub task_detail_separator: Style,
  pub task_detail_title: Style,
  pub task_detail_value: Style,

  pub task_list_icon_cancelled: Style,
  pub task_list_icon_done: Style,
  pub task_list_icon_in_progress: Style,
  pub task_list_icon_open: Style,
  pub task_list_priority: Style,
  pub task_list_title: Style,
  pub task_list_title_cancelled: Style,
}

impl Default for Theme {
  fn default() -> Self {
    Self {
      artifact_detail_label: Style::new().fg(colors::PEWTER),
      artifact_detail_separator: Style::new().fg(colors::BORDER),
      artifact_detail_value: Style::new().fg(colors::SILVER),
      artifact_list_archived_badge: Style::new().fg(colors::DIM),
      artifact_list_kind: Style::new().fg(colors::PEWTER),
      artifact_list_tag_archived: Style::new().fg(colors::DIM),
      artifact_list_title: Style::new().fg(colors::SILVER),
      artifact_list_title_archived: Style::new().fg(colors::DIM),

      banner_author: Style::new().fg(colors::SILVER).italic(),
      banner_author_name: Style::new().fg(colors::EMBER).bold(),
      banner_gradient_end: Style::new().fg(yansi::Color::Rgb(68, 169, 211)),
      banner_gradient_start: Style::new().fg(yansi::Color::Rgb(24, 178, 155)),
      banner_shadow: Style::new().fg(yansi::Color::Rgb(14, 130, 112)),
      banner_update_command: Style::new().fg(colors::SILVER),
      banner_update_hint: Style::new().fg(colors::PEWTER),
      banner_update_message: Style::new().fg(colors::AMBER),
      banner_update_version: Style::new().fg(colors::AMBER).bold(),
      banner_version: Style::new().fg(colors::SILVER),
      banner_version_date: Style::new().fg(colors::AZURE),
      banner_version_revision: Style::new().fg(colors::JADE),

      border: Style::new().fg(colors::BORDER),

      config_heading: Style::new().fg(colors::AZURE).bold().underline(),
      config_label: Style::new().fg(colors::PEWTER),
      config_no_overrides: Style::new().fg(colors::DIM),
      config_value: Style::new().fg(colors::SILVER),

      emphasis: Style::new().fg(colors::AZURE).bold(),
      error: Style::new().fg(colors::ERROR).bold(),

      id_prefix: Style::new().fg(colors::AZURE).bold(),
      id_rest: Style::new().fg(colors::PEWTER),

      indicator_blocked: Style::new().fg(colors::ERROR).bold(),
      indicator_blocked_by_id: Style::new().fg(colors::AZURE),
      indicator_blocked_by_label: Style::new().fg(colors::PEWTER),
      indicator_blocking: Style::new().fg(colors::AMBER).bold(),

      init_command_prefix: Style::new().fg(colors::BORDER),
      init_label: Style::new().fg(colors::PEWTER),
      init_section: Style::new().fg(colors::PEWTER),
      init_value: Style::new().fg(colors::SILVER),

      iteration_detail_count_blocked: Style::new().fg(colors::ERROR).bold(),
      iteration_detail_count_done: Style::new().fg(colors::JADE),
      iteration_detail_count_in_progress: Style::new().fg(colors::AMBER),
      iteration_detail_count_open: Style::new().fg(colors::SILVER),
      iteration_detail_label: Style::new().fg(colors::PEWTER),
      iteration_detail_value: Style::new().fg(colors::SILVER),

      iteration_graph_branch: Style::new().fg(colors::BORDER),
      iteration_graph_phase_icon: Style::new().fg(colors::AZURE).bold(),
      iteration_graph_phase_label: Style::new().fg(colors::AZURE).bold().underline(),
      iteration_graph_phase_name: Style::new().fg(colors::PEWTER),
      iteration_graph_separator: Style::new().fg(colors::BORDER),
      iteration_graph_title: Style::new().fg(colors::SILVER).bold(),

      iteration_list_summary: Style::new().fg(colors::PEWTER),
      iteration_list_title: Style::new().fg(colors::SILVER),

      list_heading: Style::new().fg(colors::AZURE).bold().underline(),
      list_summary: Style::new().fg(colors::PEWTER),

      log_debug: Style::new().fg(colors::AZURE_LIGHT),
      log_error: Style::new().fg(colors::ERROR),
      log_info: Style::new().fg(colors::AZURE),
      log_timestamp: Style::new().fg(colors::DIM),
      log_trace: Style::new().fg(colors::DIM),
      log_warn: Style::new().fg(colors::AMBER),

      markdown_blockquote: Style::new().fg(colors::PEWTER).italic(),
      markdown_blockquote_border: Style::new().fg(colors::DIM),
      markdown_code_block: Style::new().fg(colors::SILVER),
      markdown_code_border: Style::new().fg(colors::AZURE_DARK),
      markdown_code_inline: Style::new().fg(colors::EMBER),
      markdown_emphasis: Style::default().italic(),
      markdown_heading: Style::new().fg(colors::AZURE).bold(),
      markdown_link: Style::new().fg(colors::AZURE).underline(),
      markdown_rule: Style::new().fg(colors::BORDER),
      markdown_strong: Style::default().bold(),

      message_created_label: Style::new().fg(colors::SILVER),
      message_success_icon: Style::new().fg(colors::JADE).bold(),
      message_updated_label: Style::new().fg(colors::SILVER),

      muted: Style::new().fg(colors::PEWTER),

      search_expand_separator: Style::new().fg(colors::BORDER),
      search_no_results_hint: Style::new().fg(colors::DIM),
      search_query: Style::new().fg(colors::SILVER),
      search_summary: Style::new().fg(colors::PEWTER),
      search_type_label: Style::new().fg(colors::PEWTER),

      status_cancelled: Style::new().fg(colors::DIM),
      status_done: Style::new().fg(colors::JADE),
      status_in_progress: Style::new().fg(colors::AMBER),
      status_open: Style::new().fg(colors::SILVER),

      success: Style::new().fg(colors::JADE).bold(),
      tag: Style::new().fg(colors::AZURE).italic(),

      task_detail_label: Style::new().fg(colors::PEWTER),
      task_detail_separator: Style::new().fg(colors::BORDER),
      task_detail_title: Style::new().fg(colors::SILVER),
      task_detail_value: Style::new().fg(colors::SILVER),

      task_list_icon_cancelled: Style::new().fg(colors::DIM),
      task_list_icon_done: Style::new().fg(colors::JADE),
      task_list_icon_in_progress: Style::new().fg(colors::AMBER),
      task_list_icon_open: Style::new().fg(colors::SILVER),
      task_list_priority: Style::new().fg(colors::PEWTER),
      task_list_title: Style::new().fg(colors::SILVER),
      task_list_title_cancelled: Style::new().fg(colors::DIM),
    }
  }
}

impl Theme {
  /// Build a theme by applying palette cascades and token overrides from config.
  ///
  /// Resolution order:
  /// 1. Built-in defaults
  /// 2. Palette colors — cascade to all tokens referencing the slot (fg only, modifiers preserved)
  /// 3. Token overrides — most specific, full [`ColorValue`] wins
  pub fn from_config(settings: &crate::config::Settings) -> Self {
    let mut theme = Self::default();
    theme.apply_palette(settings.colors());
    theme.apply_overrides(settings.colors());
    theme
  }

  /// Merge user color overrides into this theme, matching dot-separated keys to fields.
  pub fn apply_overrides(&mut self, colors: &crate::config::colors::Settings) {
    for (key, value) in colors.iter() {
      if let Some(style) = self.style_mut(key) {
        *style = value.apply_to(*style);
      } else {
        log::warn!("unknown color token  key={key:?}");
      }
    }
  }

  /// Build a theme from color settings alone (palette + overrides), without full app config.
  #[cfg(test)]
  fn from_config_colors(colors: &crate::config::colors::Settings) -> Self {
    let mut theme = Self::default();
    theme.apply_palette(colors);
    theme.apply_overrides(colors);
    theme
  }

  /// Cascade palette color overrides to all tokens referencing each slot.
  ///
  /// Palette values are color-only: they replace the fg color but preserve
  /// per-token modifiers (bold, italic, etc.) from defaults.
  fn apply_palette(&mut self, colors: &crate::config::colors::Settings) {
    if colors.palette.is_empty() {
      return;
    }
    for key in super::palette::ALL_TOKEN_KEYS {
      if let Some(slot) = super::palette::palette_for_token(key)
        && let Some(&color) = colors.palette.get(slot.key())
        && let Some(style) = self.style_mut(key)
      {
        *style = style.fg(color);
      }
    }
  }

  /// Return a mutable reference to the style field for the given token key.
  fn style_mut(&mut self, key: &str) -> Option<&mut Style> {
    match key {
      "artifact.detail.label" => Some(&mut self.artifact_detail_label),
      "artifact.detail.separator" => Some(&mut self.artifact_detail_separator),
      "artifact.detail.value" => Some(&mut self.artifact_detail_value),
      "artifact.list.archived.badge" => Some(&mut self.artifact_list_archived_badge),
      "artifact.list.kind" => Some(&mut self.artifact_list_kind),
      "artifact.list.tag.archived" => Some(&mut self.artifact_list_tag_archived),
      "artifact.list.title" => Some(&mut self.artifact_list_title),
      "artifact.list.title.archived" => Some(&mut self.artifact_list_title_archived),

      "banner.author" => Some(&mut self.banner_author),
      "banner.author.name" => Some(&mut self.banner_author_name),
      "banner.gradient.end" => Some(&mut self.banner_gradient_end),
      "banner.gradient.start" => Some(&mut self.banner_gradient_start),
      "banner.shadow" => Some(&mut self.banner_shadow),
      "banner.update.command" => Some(&mut self.banner_update_command),
      "banner.update.hint" => Some(&mut self.banner_update_hint),
      "banner.update.message" => Some(&mut self.banner_update_message),
      "banner.update.version" => Some(&mut self.banner_update_version),
      "banner.version" => Some(&mut self.banner_version),
      "banner.version.date" => Some(&mut self.banner_version_date),
      "banner.version.revision" => Some(&mut self.banner_version_revision),

      "border" => Some(&mut self.border),

      "config.heading" => Some(&mut self.config_heading),
      "config.label" => Some(&mut self.config_label),
      "config.no_overrides" => Some(&mut self.config_no_overrides),
      "config.value" => Some(&mut self.config_value),

      "emphasis" => Some(&mut self.emphasis),
      "error" => Some(&mut self.error),

      "id.prefix" => Some(&mut self.id_prefix),
      "id.rest" => Some(&mut self.id_rest),

      "indicator.blocked" => Some(&mut self.indicator_blocked),
      "indicator.blocked_by.id" => Some(&mut self.indicator_blocked_by_id),
      "indicator.blocked_by.label" => Some(&mut self.indicator_blocked_by_label),
      "indicator.blocking" => Some(&mut self.indicator_blocking),

      "init.command.prefix" => Some(&mut self.init_command_prefix),
      "init.label" => Some(&mut self.init_label),
      "init.section" => Some(&mut self.init_section),
      "init.value" => Some(&mut self.init_value),

      "iteration.detail.count.blocked" => Some(&mut self.iteration_detail_count_blocked),
      "iteration.detail.count.done" => Some(&mut self.iteration_detail_count_done),
      "iteration.detail.count.in_progress" => Some(&mut self.iteration_detail_count_in_progress),
      "iteration.detail.count.open" => Some(&mut self.iteration_detail_count_open),
      "iteration.detail.label" => Some(&mut self.iteration_detail_label),
      "iteration.detail.value" => Some(&mut self.iteration_detail_value),

      "iteration.graph.branch" => Some(&mut self.iteration_graph_branch),
      "iteration.graph.phase.icon" => Some(&mut self.iteration_graph_phase_icon),
      "iteration.graph.phase.label" => Some(&mut self.iteration_graph_phase_label),
      "iteration.graph.phase.name" => Some(&mut self.iteration_graph_phase_name),
      "iteration.graph.separator" => Some(&mut self.iteration_graph_separator),
      "iteration.graph.title" => Some(&mut self.iteration_graph_title),

      "iteration.list.summary" => Some(&mut self.iteration_list_summary),
      "iteration.list.title" => Some(&mut self.iteration_list_title),

      "list.heading" => Some(&mut self.list_heading),
      "list.summary" => Some(&mut self.list_summary),

      "log.debug" => Some(&mut self.log_debug),
      "log.error" => Some(&mut self.log_error),
      "log.info" => Some(&mut self.log_info),
      "log.timestamp" => Some(&mut self.log_timestamp),
      "log.trace" => Some(&mut self.log_trace),
      "log.warn" => Some(&mut self.log_warn),

      "markdown.blockquote" | "md.blockquote" => Some(&mut self.markdown_blockquote),
      "markdown.blockquote.border" | "md.blockquote.border" => Some(&mut self.markdown_blockquote_border),
      "markdown.code.block" | "md.code.block" => Some(&mut self.markdown_code_block),
      "markdown.code.border" | "md.code.border" => Some(&mut self.markdown_code_border),
      "markdown.code.inline" | "md.code" => Some(&mut self.markdown_code_inline),
      "markdown.emphasis" | "md.emphasis" => Some(&mut self.markdown_emphasis),
      "markdown.heading" | "md.heading" => Some(&mut self.markdown_heading),
      "markdown.link" | "md.link" => Some(&mut self.markdown_link),
      "markdown.rule" | "md.rule" => Some(&mut self.markdown_rule),
      "markdown.strong" | "md.strong" => Some(&mut self.markdown_strong),

      "message.created.label" => Some(&mut self.message_created_label),
      "message.success.icon" => Some(&mut self.message_success_icon),
      "message.updated.label" => Some(&mut self.message_updated_label),

      "muted" => Some(&mut self.muted),

      "search.expand.separator" => Some(&mut self.search_expand_separator),
      "search.no_results.hint" => Some(&mut self.search_no_results_hint),
      "search.query" => Some(&mut self.search_query),
      "search.summary" => Some(&mut self.search_summary),
      "search.type.label" => Some(&mut self.search_type_label),

      "status.cancelled" => Some(&mut self.status_cancelled),
      "status.done" => Some(&mut self.status_done),
      "status.in_progress" => Some(&mut self.status_in_progress),
      "status.open" => Some(&mut self.status_open),

      "success" => Some(&mut self.success),
      "tag" => Some(&mut self.tag),

      "task.detail.label" => Some(&mut self.task_detail_label),
      "task.detail.separator" => Some(&mut self.task_detail_separator),
      "task.detail.title" => Some(&mut self.task_detail_title),
      "task.detail.value" => Some(&mut self.task_detail_value),

      "task.list.icon.cancelled" => Some(&mut self.task_list_icon_cancelled),
      "task.list.icon.done" => Some(&mut self.task_list_icon_done),
      "task.list.icon.in_progress" => Some(&mut self.task_list_icon_in_progress),
      "task.list.icon.open" => Some(&mut self.task_list_icon_open),
      "task.list.priority" => Some(&mut self.task_list_priority),
      "task.list.title" => Some(&mut self.task_list_title),
      "task.list.title.cancelled" => Some(&mut self.task_list_title_cancelled),

      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use yansi::{Color, Paint};

  use super::*;

  #[test]
  fn it_cascades_palette_to_all_tokens_referencing_slot() {
    let purple = Color::Rgb(148, 72, 199);
    let mut colors = crate::config::colors::Settings::default();
    colors.palette.insert("primary".to_string(), purple);

    let mut theme = Theme::default();
    theme.apply_palette(&colors);

    // All Primary tokens should now have purple fg but keep their original modifiers
    assert_eq!(
      format!("{:?}", theme.emphasis),
      format!("{:?}", Style::new().fg(purple).bold())
    );
    assert_eq!(
      format!("{:?}", theme.config_heading),
      format!("{:?}", Style::new().fg(purple).bold().underline())
    );
    assert_eq!(
      format!("{:?}", theme.tag),
      format!("{:?}", Style::new().fg(purple).italic())
    );
    assert_eq!(
      format!("{:?}", theme.log_info),
      format!("{:?}", Style::new().fg(purple))
    );
  }

  #[test]
  fn it_creates_successfully_with_defaults() {
    let theme = Theme::default();
    let _ = theme.emphasis;
  }

  #[test]
  fn it_does_not_affect_inline_rgb_tokens_on_palette_change() {
    let mut colors = crate::config::colors::Settings::default();
    colors.palette.insert("primary".to_string(), Color::Rgb(255, 0, 0));

    let mut theme = Theme::default();
    let original_gradient = theme.banner_gradient_start;
    theme.apply_palette(&colors);

    assert_eq!(
      format!("{:?}", theme.banner_gradient_start),
      format!("{:?}", original_gradient)
    );
  }

  #[test]
  fn it_does_not_affect_modifier_only_tokens_on_palette_change() {
    let mut colors = crate::config::colors::Settings::default();
    colors.palette.insert("primary".to_string(), Color::Rgb(255, 0, 0));

    let mut theme = Theme::default();
    theme.apply_palette(&colors);

    assert_eq!(
      format!("{:?}", theme.markdown_emphasis),
      format!("{:?}", Style::default().italic())
    );
    assert_eq!(
      format!("{:?}", theme.markdown_strong),
      format!("{:?}", Style::default().bold())
    );
  }

  #[test]
  fn it_lets_token_override_win_over_palette() {
    let purple = Color::Rgb(148, 72, 199);
    let green = Color::Rgb(0, 255, 0);
    let mut colors = crate::config::colors::Settings::default();
    colors.palette.insert("primary".to_string(), purple);
    colors.overrides.insert(
      "emphasis".to_string(),
      crate::config::colors::ColorValue {
        bg: None,
        bold: false,
        dim: false,
        fg: Some(green),
        italic: true,
        underline: false,
      },
    );

    let theme = Theme::from_config_colors(&colors);

    // emphasis should have green fg + italic from override, bold preserved from default
    assert_eq!(
      format!("{:?}", theme.emphasis),
      format!("{:?}", Style::new().fg(green).bold().italic())
    );
    // but other primary tokens should have purple from palette
    assert_eq!(
      format!("{:?}", theme.tag),
      format!("{:?}", Style::new().fg(purple).italic())
    );
  }

  #[test]
  fn it_preserves_modifiers_when_palette_cascades() {
    let red = Color::Rgb(255, 0, 0);
    let mut colors = crate::config::colors::Settings::default();
    colors.palette.insert("success".to_string(), red);

    let mut theme = Theme::default();
    theme.apply_palette(&colors);

    // success token has .bold() modifier — should be preserved
    assert_eq!(
      format!("{:?}", theme.success),
      format!("{:?}", Style::new().fg(red).bold())
    );
    // status_done has no modifiers — just fg
    assert_eq!(
      format!("{:?}", theme.status_done),
      format!("{:?}", Style::new().fg(red))
    );
  }

  #[test]
  fn it_returns_default_when_no_overrides_from_config() {
    let settings = crate::config::Settings::default();
    let from_cfg = Theme::from_config(&settings);
    let default = Theme::default();

    assert_eq!(format!("{:?}", from_cfg.emphasis), format!("{:?}", default.emphasis));
    assert_eq!(format!("{:?}", from_cfg.log_error), format!("{:?}", default.log_error));
  }

  #[test]
  fn it_styles_emphasis_as_azure_bold() {
    let theme = Theme::default();
    let styled = "x".paint(theme.emphasis);
    let rendered = format!("{styled}");
    assert!(rendered.contains('x'));
  }

  #[test]
  fn it_uses_default_fg_for_markdown_emphasis() {
    let theme = Theme::default();
    let expected = Style::default().italic();
    assert_eq!(format!("{:?}", theme.markdown_emphasis), format!("{:?}", expected),);
  }

  #[test]
  fn it_uses_default_fg_for_markdown_strong() {
    let theme = Theme::default();
    let expected = Style::default().bold();
    assert_eq!(format!("{:?}", theme.markdown_strong), format!("{:?}", expected),);
  }

  #[test]
  fn it_uses_inline_rgb_for_banner_gradient_start() {
    let theme = Theme::default();
    let expected = Style::new().fg(Color::Rgb(24, 178, 155));

    assert_eq!(format!("{:?}", theme.banner_gradient_start), format!("{:?}", expected),);
  }
}
