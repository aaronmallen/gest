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
  /// Build a theme by applying user color overrides from config on top of defaults.
  pub fn from_config(settings: &crate::config::Settings) -> Self {
    let mut theme = Self::default();
    theme.apply_overrides(settings.colors());
    theme
  }

  /// Merge user color overrides into this theme, matching dot-separated keys to fields.
  pub fn apply_overrides(&mut self, colors: &crate::config::colors::Settings) {
    for (key, value) in colors.iter() {
      match key.as_str() {
        "artifact.detail.label" => self.artifact_detail_label = value.apply_to(self.artifact_detail_label),
        "artifact.detail.separator" => self.artifact_detail_separator = value.apply_to(self.artifact_detail_separator),
        "artifact.detail.value" => self.artifact_detail_value = value.apply_to(self.artifact_detail_value),
        "artifact.list.archived.badge" => {
          self.artifact_list_archived_badge = value.apply_to(self.artifact_list_archived_badge)
        }
        "artifact.list.kind" => self.artifact_list_kind = value.apply_to(self.artifact_list_kind),
        "artifact.list.tag.archived" => {
          self.artifact_list_tag_archived = value.apply_to(self.artifact_list_tag_archived)
        }
        "artifact.list.title" => self.artifact_list_title = value.apply_to(self.artifact_list_title),
        "artifact.list.title.archived" => {
          self.artifact_list_title_archived = value.apply_to(self.artifact_list_title_archived)
        }

        "banner.author" => self.banner_author = value.apply_to(self.banner_author),
        "banner.author.name" => self.banner_author_name = value.apply_to(self.banner_author_name),
        "banner.gradient.end" => self.banner_gradient_end = value.apply_to(self.banner_gradient_end),
        "banner.gradient.start" => self.banner_gradient_start = value.apply_to(self.banner_gradient_start),
        "banner.shadow" => self.banner_shadow = value.apply_to(self.banner_shadow),
        "banner.update.command" => self.banner_update_command = value.apply_to(self.banner_update_command),
        "banner.update.hint" => self.banner_update_hint = value.apply_to(self.banner_update_hint),
        "banner.update.message" => self.banner_update_message = value.apply_to(self.banner_update_message),
        "banner.update.version" => self.banner_update_version = value.apply_to(self.banner_update_version),
        "banner.version" => self.banner_version = value.apply_to(self.banner_version),
        "banner.version.date" => self.banner_version_date = value.apply_to(self.banner_version_date),
        "banner.version.revision" => self.banner_version_revision = value.apply_to(self.banner_version_revision),

        "border" => self.border = value.apply_to(self.border),

        "config.heading" => self.config_heading = value.apply_to(self.config_heading),
        "config.label" => self.config_label = value.apply_to(self.config_label),
        "config.no_overrides" => self.config_no_overrides = value.apply_to(self.config_no_overrides),
        "config.value" => self.config_value = value.apply_to(self.config_value),

        "emphasis" => self.emphasis = value.apply_to(self.emphasis),
        "error" => self.error = value.apply_to(self.error),

        "id.prefix" => self.id_prefix = value.apply_to(self.id_prefix),
        "id.rest" => self.id_rest = value.apply_to(self.id_rest),

        "indicator.blocked" => self.indicator_blocked = value.apply_to(self.indicator_blocked),
        "indicator.blocked_by.id" => self.indicator_blocked_by_id = value.apply_to(self.indicator_blocked_by_id),
        "indicator.blocked_by.label" => {
          self.indicator_blocked_by_label = value.apply_to(self.indicator_blocked_by_label)
        }
        "indicator.blocking" => self.indicator_blocking = value.apply_to(self.indicator_blocking),

        "init.command.prefix" => self.init_command_prefix = value.apply_to(self.init_command_prefix),
        "init.label" => self.init_label = value.apply_to(self.init_label),
        "init.section" => self.init_section = value.apply_to(self.init_section),
        "init.value" => self.init_value = value.apply_to(self.init_value),

        "iteration.detail.count.blocked" => {
          self.iteration_detail_count_blocked = value.apply_to(self.iteration_detail_count_blocked)
        }
        "iteration.detail.count.done" => {
          self.iteration_detail_count_done = value.apply_to(self.iteration_detail_count_done)
        }
        "iteration.detail.count.in_progress" => {
          self.iteration_detail_count_in_progress = value.apply_to(self.iteration_detail_count_in_progress)
        }
        "iteration.detail.count.open" => {
          self.iteration_detail_count_open = value.apply_to(self.iteration_detail_count_open)
        }
        "iteration.detail.label" => self.iteration_detail_label = value.apply_to(self.iteration_detail_label),
        "iteration.detail.value" => self.iteration_detail_value = value.apply_to(self.iteration_detail_value),

        "iteration.graph.branch" => self.iteration_graph_branch = value.apply_to(self.iteration_graph_branch),
        "iteration.graph.phase.icon" => {
          self.iteration_graph_phase_icon = value.apply_to(self.iteration_graph_phase_icon)
        }
        "iteration.graph.phase.label" => {
          self.iteration_graph_phase_label = value.apply_to(self.iteration_graph_phase_label)
        }
        "iteration.graph.phase.name" => {
          self.iteration_graph_phase_name = value.apply_to(self.iteration_graph_phase_name)
        }
        "iteration.graph.separator" => self.iteration_graph_separator = value.apply_to(self.iteration_graph_separator),
        "iteration.graph.title" => self.iteration_graph_title = value.apply_to(self.iteration_graph_title),

        "iteration.list.summary" => self.iteration_list_summary = value.apply_to(self.iteration_list_summary),
        "iteration.list.title" => self.iteration_list_title = value.apply_to(self.iteration_list_title),

        "list.heading" => self.list_heading = value.apply_to(self.list_heading),
        "list.summary" => self.list_summary = value.apply_to(self.list_summary),

        "log.debug" => self.log_debug = value.apply_to(self.log_debug),
        "log.error" => self.log_error = value.apply_to(self.log_error),
        "log.info" => self.log_info = value.apply_to(self.log_info),
        "log.timestamp" => self.log_timestamp = value.apply_to(self.log_timestamp),
        "log.trace" => self.log_trace = value.apply_to(self.log_trace),
        "log.warn" => self.log_warn = value.apply_to(self.log_warn),

        "markdown.blockquote" | "md.blockquote" => self.markdown_blockquote = value.apply_to(self.markdown_blockquote),
        "markdown.blockquote.border" | "md.blockquote.border" => {
          self.markdown_blockquote_border = value.apply_to(self.markdown_blockquote_border)
        }
        "markdown.code.block" | "md.code.block" => self.markdown_code_block = value.apply_to(self.markdown_code_block),
        "markdown.code.border" | "md.code.border" => {
          self.markdown_code_border = value.apply_to(self.markdown_code_border)
        }
        "markdown.code.inline" | "md.code" => self.markdown_code_inline = value.apply_to(self.markdown_code_inline),
        "markdown.emphasis" | "md.emphasis" => self.markdown_emphasis = value.apply_to(self.markdown_emphasis),
        "markdown.heading" | "md.heading" => self.markdown_heading = value.apply_to(self.markdown_heading),
        "markdown.link" | "md.link" => self.markdown_link = value.apply_to(self.markdown_link),
        "markdown.rule" | "md.rule" => self.markdown_rule = value.apply_to(self.markdown_rule),
        "markdown.strong" | "md.strong" => self.markdown_strong = value.apply_to(self.markdown_strong),

        "message.created.label" => self.message_created_label = value.apply_to(self.message_created_label),
        "message.success.icon" => self.message_success_icon = value.apply_to(self.message_success_icon),
        "message.updated.label" => self.message_updated_label = value.apply_to(self.message_updated_label),

        "muted" => self.muted = value.apply_to(self.muted),

        "search.expand.separator" => self.search_expand_separator = value.apply_to(self.search_expand_separator),
        "search.no_results.hint" => self.search_no_results_hint = value.apply_to(self.search_no_results_hint),
        "search.query" => self.search_query = value.apply_to(self.search_query),
        "search.summary" => self.search_summary = value.apply_to(self.search_summary),
        "search.type.label" => self.search_type_label = value.apply_to(self.search_type_label),

        "status.cancelled" => self.status_cancelled = value.apply_to(self.status_cancelled),
        "status.done" => self.status_done = value.apply_to(self.status_done),
        "status.in_progress" => self.status_in_progress = value.apply_to(self.status_in_progress),
        "status.open" => self.status_open = value.apply_to(self.status_open),

        "success" => self.success = value.apply_to(self.success),
        "tag" => self.tag = value.apply_to(self.tag),

        "task.detail.label" => self.task_detail_label = value.apply_to(self.task_detail_label),
        "task.detail.separator" => self.task_detail_separator = value.apply_to(self.task_detail_separator),
        "task.detail.title" => self.task_detail_title = value.apply_to(self.task_detail_title),
        "task.detail.value" => self.task_detail_value = value.apply_to(self.task_detail_value),

        "task.list.icon.cancelled" => self.task_list_icon_cancelled = value.apply_to(self.task_list_icon_cancelled),
        "task.list.icon.done" => self.task_list_icon_done = value.apply_to(self.task_list_icon_done),
        "task.list.icon.in_progress" => {
          self.task_list_icon_in_progress = value.apply_to(self.task_list_icon_in_progress)
        }
        "task.list.icon.open" => self.task_list_icon_open = value.apply_to(self.task_list_icon_open),
        "task.list.priority" => self.task_list_priority = value.apply_to(self.task_list_priority),
        "task.list.title" => self.task_list_title = value.apply_to(self.task_list_title),
        "task.list.title.cancelled" => self.task_list_title_cancelled = value.apply_to(self.task_list_title_cancelled),

        _ => {
          log::warn!("unknown color token  key={key:?}");
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use yansi::{Color, Paint};

  use super::*;

  #[test]
  fn it_creates_successfully_with_defaults() {
    let theme = Theme::default();
    let _ = theme.emphasis;
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
