//! Semantic style tokens and theme resolution.
//!
//! A [`Theme`] holds one [`Style`] per UI token. Tokens are resolved in three
//! layers: built-in defaults, palette overrides, and per-token overrides from
//! the user's configuration.

use std::sync::OnceLock;

use Palette::*;
use getset::Getters;
use yansi::{Color, Style};

/// Process-wide resolved theme, set once during startup.
static THEME: OnceLock<Theme> = OnceLock::new();

/// Every known token key, used to drive palette cascading.
pub const ALL_TOKENS: &[&str] = &[
  "artifact.detail.label",
  "artifact.detail.separator",
  "artifact.detail.value",
  "artifact.list.archived.badge",
  "artifact.list.tag.archived",
  "artifact.list.title",
  "artifact.list.title.archived",
  "banner.author",
  "banner.author.name",
  "banner.gradient.end",
  "banner.gradient.start",
  "banner.shadow",
  "banner.update.command",
  "banner.update.hint",
  "banner.update.message",
  "banner.update.version",
  "banner.version",
  "banner.version.date",
  "banner.version.revision",
  "border",
  "config.heading",
  "config.label",
  "config.no_overrides",
  "config.value",
  "emphasis",
  "error",
  "id.prefix",
  "id.rest",
  "indicator.blocked",
  "indicator.blocked_by.id",
  "indicator.blocked_by.label",
  "indicator.blocking",
  "init.command.prefix",
  "init.label",
  "init.section",
  "init.value",
  "iteration.detail.count.blocked",
  "iteration.detail.count.done",
  "iteration.detail.count.in_progress",
  "iteration.detail.count.open",
  "iteration.detail.label",
  "iteration.detail.value",
  "iteration.graph.branch",
  "iteration.graph.phase.icon",
  "iteration.graph.phase.label",
  "iteration.graph.phase.name",
  "iteration.graph.separator",
  "iteration.graph.title",
  "iteration.list.summary",
  "iteration.list.title",
  "iteration.status.label",
  "iteration.status.progress",
  "iteration.status.value",
  "list.heading",
  "list.summary",
  "log.debug",
  "log.error",
  "log.info",
  "log.timestamp",
  "log.trace",
  "log.warn",
  "markdown.alert.caution.border",
  "markdown.alert.important.border",
  "markdown.alert.note.border",
  "markdown.alert.tip.border",
  "markdown.alert.warning.border",
  "markdown.blockquote",
  "markdown.blockquote.border",
  "markdown.code.block",
  "markdown.code.border",
  "markdown.code.inline",
  "markdown.emphasis",
  "markdown.heading",
  "markdown.link",
  "markdown.rule",
  "markdown.strong",
  "meta.not_set",
  "meta.value",
  "message.created.label",
  "message.success.icon",
  "message.updated.label",
  "migrate.count",
  "muted",
  "note.detail.label",
  "note.detail.separator",
  "note.detail.value",
  "note.list.body",
  "note.list.id",
  "project.list.root",
  "project.show.value",
  "search.expand.separator",
  "search.no_results.hint",
  "search.query",
  "search.summary",
  "search.type.label",
  "serve.url",
  "status.cancelled",
  "status.done",
  "status.in_progress",
  "status.open",
  "success",
  "tag",
  "tag.list.count",
  "tag.list.heading",
  "task.detail.label",
  "task.detail.separator",
  "task.detail.title",
  "task.detail.value",
  "task.list.icon.cancelled",
  "task.list.icon.done",
  "task.list.icon.in_progress",
  "task.list.icon.open",
  "task.list.priority",
  "task.list.title",
  "task.list.title.cancelled",
];

/// Semantic palette slots that style tokens can reference.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Palette {
  Accent,
  Border,
  Error,
  Primary,
  PrimaryDark,
  PrimaryLight,
  Success,
  Text,
  TextDim,
  TextMuted,
  Warning,
}

impl Palette {
  /// All palette variants in definition order.
  pub const ALL: [Palette; 11] = [
    Self::Accent,
    Self::Border,
    Self::Error,
    Self::Primary,
    Self::PrimaryDark,
    Self::PrimaryLight,
    Self::Success,
    Self::Text,
    Self::TextDim,
    Self::TextMuted,
    Self::Warning,
  ];

  /// The built-in default color for this palette slot.
  pub fn default_color(self) -> Color {
    match self {
      Self::Accent => Color::Rgb(208, 88, 48),         // ember
      Self::Border => Color::Rgb(48, 50, 58),          // border
      Self::Error => Color::Rgb(208, 56, 56),          // error
      Self::Primary => Color::Rgb(78, 168, 224),       // azure
      Self::PrimaryDark => Color::Rgb(50, 120, 176),   // azure dark
      Self::PrimaryLight => Color::Rgb(124, 196, 240), // azure light
      Self::Success => Color::Rgb(54, 190, 120),       // jade
      Self::Text => Color::Rgb(196, 200, 212),         // silver
      Self::TextDim => Color::Rgb(88, 94, 110),        // dim
      Self::TextMuted => Color::Rgb(124, 130, 148),    // pewter
      Self::Warning => Color::Rgb(204, 152, 32),       // amber
    }
  }

  /// The config key used in `[colors.palette]` for this slot.
  pub fn key(self) -> &'static str {
    match self {
      Self::Accent => "accent",
      Self::Border => "border",
      Self::Error => "error",
      Self::Primary => "primary",
      Self::PrimaryDark => "primary.dark",
      Self::PrimaryLight => "primary.light",
      Self::Success => "success",
      Self::Text => "text",
      Self::TextDim => "text.dim",
      Self::TextMuted => "text.muted",
      Self::Warning => "warning",
    }
  }
}

/// Semantic style tokens for all UI elements.
///
/// Each field maps a named UI role to a [`Style`]. The [`Default`]
/// implementation uses the brand palette.
#[derive(Clone, Debug, Getters)]
pub struct Theme {
  #[get = "pub"]
  artifact_detail_label: Style,
  #[get = "pub"]
  artifact_detail_separator: Style,
  #[get = "pub"]
  artifact_detail_value: Style,
  #[get = "pub"]
  artifact_list_archived_badge: Style,
  #[get = "pub"]
  artifact_list_tag_archived: Style,
  #[get = "pub"]
  artifact_list_title: Style,
  #[get = "pub"]
  artifact_list_title_archived: Style,

  #[get = "pub"]
  banner_author: Style,
  #[get = "pub"]
  banner_author_name: Style,
  #[get = "pub"]
  banner_gradient_end: Style,
  #[get = "pub"]
  banner_gradient_start: Style,
  #[get = "pub"]
  banner_shadow: Style,
  #[get = "pub"]
  banner_update_command: Style,
  #[get = "pub"]
  banner_update_hint: Style,
  #[get = "pub"]
  banner_update_message: Style,
  #[get = "pub"]
  banner_update_version: Style,
  #[get = "pub"]
  banner_version: Style,
  #[get = "pub"]
  banner_version_date: Style,
  #[get = "pub"]
  banner_version_revision: Style,

  #[get = "pub"]
  border: Style,

  #[get = "pub"]
  config_heading: Style,
  #[get = "pub"]
  config_label: Style,
  #[get = "pub"]
  config_no_overrides: Style,
  #[get = "pub"]
  config_value: Style,

  #[get = "pub"]
  emphasis: Style,
  #[get = "pub"]
  error: Style,

  #[get = "pub"]
  id_prefix: Style,
  #[get = "pub"]
  id_rest: Style,

  #[get = "pub"]
  indicator_blocked: Style,
  #[get = "pub"]
  indicator_blocked_by_id: Style,
  #[get = "pub"]
  indicator_blocked_by_label: Style,
  #[get = "pub"]
  indicator_blocking: Style,

  #[get = "pub"]
  init_command_prefix: Style,
  #[get = "pub"]
  init_label: Style,
  #[get = "pub"]
  init_section: Style,
  #[get = "pub"]
  init_value: Style,

  #[get = "pub"]
  iteration_detail_count_blocked: Style,
  #[get = "pub"]
  iteration_detail_count_done: Style,
  #[get = "pub"]
  iteration_detail_count_in_progress: Style,
  #[get = "pub"]
  iteration_detail_count_open: Style,
  #[get = "pub"]
  iteration_detail_label: Style,
  #[get = "pub"]
  iteration_detail_value: Style,

  #[get = "pub"]
  iteration_graph_branch: Style,
  #[get = "pub"]
  iteration_graph_phase_icon: Style,
  #[get = "pub"]
  iteration_graph_phase_label: Style,
  #[get = "pub"]
  iteration_graph_phase_name: Style,
  #[get = "pub"]
  iteration_graph_separator: Style,

  #[get = "pub"]
  iteration_graph_title: Style,

  #[get = "pub"]
  iteration_list_summary: Style,
  #[get = "pub"]
  iteration_list_title: Style,

  #[get = "pub"]
  iteration_status_label: Style,
  #[get = "pub"]
  iteration_status_progress: Style,
  #[get = "pub"]
  iteration_status_value: Style,

  #[get = "pub"]
  list_heading: Style,
  #[get = "pub"]
  list_summary: Style,

  #[get = "pub"]
  log_debug: Style,
  #[get = "pub"]
  log_error: Style,
  #[get = "pub"]
  log_info: Style,
  #[get = "pub"]
  log_timestamp: Style,
  #[get = "pub"]
  log_trace: Style,
  #[get = "pub"]
  log_warn: Style,

  #[get = "pub"]
  markdown_alert_caution_border: Style,
  #[get = "pub"]
  markdown_alert_important_border: Style,
  #[get = "pub"]
  markdown_alert_note_border: Style,
  #[get = "pub"]
  markdown_alert_tip_border: Style,
  #[get = "pub"]
  markdown_alert_warning_border: Style,
  #[get = "pub"]
  markdown_blockquote: Style,
  #[get = "pub"]
  markdown_blockquote_border: Style,
  #[get = "pub"]
  markdown_code_block: Style,
  #[get = "pub"]
  markdown_code_border: Style,
  #[get = "pub"]
  markdown_code_inline: Style,
  #[get = "pub"]
  markdown_emphasis: Style,
  #[get = "pub"]
  markdown_heading: Style,
  #[get = "pub"]
  markdown_link: Style,
  #[get = "pub"]
  markdown_rule: Style,
  #[get = "pub"]
  markdown_strong: Style,

  #[get = "pub"]
  meta_not_set: Style,
  #[get = "pub"]
  meta_value: Style,

  #[get = "pub"]
  message_created_label: Style,
  #[get = "pub"]
  message_success_icon: Style,
  #[get = "pub"]
  message_updated_label: Style,

  #[get = "pub"]
  migrate_count: Style,

  #[get = "pub"]
  muted: Style,

  #[get = "pub"]
  note_detail_label: Style,
  #[get = "pub"]
  note_detail_separator: Style,
  #[get = "pub"]
  note_detail_value: Style,
  #[get = "pub"]
  note_list_body: Style,
  #[get = "pub"]
  note_list_id: Style,

  #[get = "pub"]
  project_list_root: Style,
  #[get = "pub"]
  project_show_value: Style,

  #[get = "pub"]
  search_expand_separator: Style,
  #[get = "pub"]
  search_no_results_hint: Style,
  #[get = "pub"]
  search_query: Style,
  #[get = "pub"]
  search_summary: Style,
  #[get = "pub"]
  search_type_label: Style,

  #[get = "pub"]
  serve_url: Style,

  #[get = "pub"]
  status_cancelled: Style,
  #[get = "pub"]
  status_done: Style,
  #[get = "pub"]
  status_in_progress: Style,
  #[get = "pub"]
  status_open: Style,

  #[get = "pub"]
  success: Style,
  #[get = "pub"]
  tag: Style,
  #[get = "pub"]
  tag_list_count: Style,
  #[get = "pub"]
  tag_list_heading: Style,

  #[get = "pub"]
  task_detail_label: Style,
  #[get = "pub"]
  task_detail_separator: Style,
  #[get = "pub"]
  task_detail_title: Style,
  #[get = "pub"]
  task_detail_value: Style,

  #[get = "pub"]
  task_list_icon_cancelled: Style,
  #[get = "pub"]
  task_list_icon_done: Style,
  #[get = "pub"]
  task_list_icon_in_progress: Style,
  #[get = "pub"]
  task_list_icon_open: Style,
  #[get = "pub"]
  task_list_priority: Style,
  #[get = "pub"]
  task_list_title: Style,
  #[get = "pub"]
  task_list_title_cancelled: Style,
}

impl Default for Theme {
  fn default() -> Self {
    let c = Palette::default_color;

    Self {
      artifact_detail_label: Style::new().fg(c(TextMuted)),
      artifact_detail_separator: Style::new().fg(c(Border)),
      artifact_detail_value: Style::new().fg(c(Text)),
      artifact_list_archived_badge: Style::new().fg(c(TextDim)),
      artifact_list_tag_archived: Style::new().fg(c(TextDim)),
      artifact_list_title: Style::new().fg(c(Text)),
      artifact_list_title_archived: Style::new().fg(c(TextDim)),

      banner_author: Style::new().fg(c(Text)).italic(),
      banner_author_name: Style::new().fg(c(Accent)).bold(),
      banner_gradient_end: Style::new().fg(Color::Rgb(68, 169, 211)),
      banner_gradient_start: Style::new().fg(Color::Rgb(24, 178, 155)),
      banner_shadow: Style::new().fg(Color::Rgb(14, 130, 112)),
      banner_update_command: Style::new().fg(c(Text)),
      banner_update_hint: Style::new().fg(c(TextMuted)),
      banner_update_message: Style::new().fg(c(Warning)),
      banner_update_version: Style::new().fg(c(Warning)).bold(),
      banner_version: Style::new().fg(c(Text)),
      banner_version_date: Style::new().fg(c(Primary)),
      banner_version_revision: Style::new().fg(c(Success)),

      border: Style::new().fg(c(Border)),

      config_heading: Style::new().fg(c(Primary)).bold().underline(),
      config_label: Style::new().fg(c(TextMuted)),
      config_no_overrides: Style::new().fg(c(TextDim)),
      config_value: Style::new().fg(c(Text)),

      emphasis: Style::new().fg(c(Primary)).bold(),
      error: Style::new().fg(c(Error)).bold(),

      id_prefix: Style::new().fg(c(Primary)).bold(),
      id_rest: Style::new().fg(c(TextMuted)),

      indicator_blocked: Style::new().fg(c(Error)).bold(),
      indicator_blocked_by_id: Style::new().fg(c(Primary)),
      indicator_blocked_by_label: Style::new().fg(c(TextMuted)),
      indicator_blocking: Style::new().fg(c(Warning)).bold(),

      init_command_prefix: Style::new().fg(c(Border)),
      init_label: Style::new().fg(c(TextMuted)),
      init_section: Style::new().fg(c(TextMuted)),
      init_value: Style::new().fg(c(Text)),

      iteration_detail_count_blocked: Style::new().fg(c(Error)).bold(),
      iteration_detail_count_done: Style::new().fg(c(Success)),
      iteration_detail_count_in_progress: Style::new().fg(c(Warning)),
      iteration_detail_count_open: Style::new().fg(c(Text)),
      iteration_detail_label: Style::new().fg(c(TextMuted)),
      iteration_detail_value: Style::new().fg(c(Text)),

      iteration_graph_branch: Style::new().fg(c(Border)),
      iteration_graph_phase_icon: Style::new().fg(c(Primary)).bold(),
      iteration_graph_phase_label: Style::new().fg(c(Primary)).bold().underline(),
      iteration_graph_phase_name: Style::new().fg(c(TextMuted)),
      iteration_graph_separator: Style::new().fg(c(Border)),

      iteration_graph_title: Style::new().fg(c(Text)).bold(),

      iteration_list_summary: Style::new().fg(c(TextMuted)),
      iteration_list_title: Style::new().fg(c(Text)),

      iteration_status_label: Style::new().fg(c(TextMuted)),
      iteration_status_progress: Style::new().fg(c(Primary)),
      iteration_status_value: Style::new().fg(c(Text)),

      list_heading: Style::new().fg(c(Primary)).bold().underline(),
      list_summary: Style::new().fg(c(TextMuted)),

      log_debug: Style::new().fg(c(PrimaryLight)),
      log_error: Style::new().fg(c(Error)),
      log_info: Style::new().fg(c(Primary)),
      log_timestamp: Style::new().fg(c(TextDim)),
      log_trace: Style::new().fg(c(TextDim)),
      log_warn: Style::new().fg(c(Warning)),

      markdown_alert_caution_border: Style::new().fg(c(Error)),
      markdown_alert_important_border: Style::new().fg(Color::Rgb(155, 89, 182)),
      markdown_alert_note_border: Style::new().fg(c(Primary)),
      markdown_alert_tip_border: Style::new().fg(c(Success)),
      markdown_alert_warning_border: Style::new().fg(c(Warning)),
      markdown_blockquote: Style::new().fg(c(TextMuted)).italic(),
      markdown_blockquote_border: Style::new().fg(c(TextDim)),
      markdown_code_block: Style::new().fg(c(Text)),
      markdown_code_border: Style::new().fg(c(PrimaryDark)),
      markdown_code_inline: Style::new().fg(c(Accent)),
      markdown_emphasis: Style::default().italic(),
      markdown_heading: Style::new().fg(c(Primary)).bold().underline(),
      markdown_link: Style::new().fg(c(Primary)).underline(),
      markdown_rule: Style::new().fg(c(Border)),
      markdown_strong: Style::default().bold(),

      meta_not_set: Style::new().fg(c(TextDim)).italic(),
      meta_value: Style::new().fg(c(Text)),

      message_created_label: Style::new().fg(c(Text)),
      message_success_icon: Style::new().fg(c(Success)).bold(),
      message_updated_label: Style::new().fg(c(Text)),

      migrate_count: Style::new().fg(c(Primary)).bold(),

      muted: Style::new().fg(c(TextMuted)),

      note_detail_label: Style::new().fg(c(TextMuted)),
      note_detail_separator: Style::new().fg(c(Border)),
      note_detail_value: Style::new().fg(c(Text)),
      note_list_body: Style::new().fg(c(TextMuted)),
      note_list_id: Style::new().fg(c(Primary)),

      project_list_root: Style::new().fg(c(Text)),
      project_show_value: Style::new().fg(c(Text)),

      search_expand_separator: Style::new().fg(c(Border)),
      search_no_results_hint: Style::new().fg(c(TextDim)),
      search_query: Style::new().fg(c(Text)),
      search_summary: Style::new().fg(c(TextMuted)),
      search_type_label: Style::new().fg(c(TextMuted)),

      serve_url: Style::new().fg(c(Primary)).underline(),

      status_cancelled: Style::new().fg(c(TextDim)),
      status_done: Style::new().fg(c(Success)),
      status_in_progress: Style::new().fg(c(Warning)),
      status_open: Style::new().fg(c(Text)),

      success: Style::new().fg(c(Success)).bold(),
      tag: Style::new().fg(c(Primary)).italic(),
      tag_list_count: Style::new().fg(c(TextMuted)),
      tag_list_heading: Style::new().fg(c(Primary)).bold().underline(),

      task_detail_label: Style::new().fg(c(TextMuted)),
      task_detail_separator: Style::new().fg(c(Border)),
      task_detail_title: Style::new().fg(c(Text)),
      task_detail_value: Style::new().fg(c(Text)),

      task_list_icon_cancelled: Style::new().fg(c(TextDim)),
      task_list_icon_done: Style::new().fg(c(Success)),
      task_list_icon_in_progress: Style::new().fg(c(Warning)),
      task_list_icon_open: Style::new().fg(c(Text)),
      task_list_priority: Style::new().fg(c(TextMuted)),
      task_list_title: Style::new().fg(c(Text)),
      task_list_title_cancelled: Style::new().fg(c(TextDim)),
    }
  }
}

impl Theme {
  /// Build a theme by applying palette cascades and token overrides from config.
  ///
  /// Resolution order:
  /// 1. Built-in defaults
  /// 2. Palette colors — cascade to all tokens referencing the slot (fg only, modifiers preserved)
  /// 3. Token overrides — most specific, full [`ColorValue`](crate::config::colors::ColorValue) wins
  pub fn from_config(settings: &crate::config::Settings) -> Self {
    let mut theme = Self::default();
    theme.apply_palette(settings.colors());
    theme.apply_overrides(settings.colors());
    theme
  }

  /// Merge user color overrides into this theme, matching dot-separated keys to fields.
  fn apply_overrides(&mut self, colors: &crate::config::colors::Settings) {
    for (key, value) in colors.iter() {
      if let Some(style) = self.style_mut(key) {
        *style = value.apply_to(*style);
      } else {
        log::warn!("unknown color token `{key:?}`");
      }
    }
  }

  /// Cascade palette color overrides to all tokens referencing each slot.
  ///
  /// Palette values are color-only: they replace the fg color but preserve
  /// per-token modifiers (bold, italic, etc.) from defaults.
  fn apply_palette(&mut self, colors: &crate::config::colors::Settings) {
    if colors.palette().is_empty() {
      return;
    }
    for key in ALL_TOKENS {
      if let Some(slot) = palette_for_token(key)
        && let Some(&color) = colors.palette().get(slot.key())
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

      "iteration.status.label" => Some(&mut self.iteration_status_label),
      "iteration.status.progress" => Some(&mut self.iteration_status_progress),
      "iteration.status.value" => Some(&mut self.iteration_status_value),

      "list.heading" => Some(&mut self.list_heading),
      "list.summary" => Some(&mut self.list_summary),

      "log.debug" => Some(&mut self.log_debug),
      "log.error" => Some(&mut self.log_error),
      "log.info" => Some(&mut self.log_info),
      "log.timestamp" => Some(&mut self.log_timestamp),
      "log.trace" => Some(&mut self.log_trace),
      "log.warn" => Some(&mut self.log_warn),

      "markdown.alert.caution.border" | "md.alert.caution.border" => Some(&mut self.markdown_alert_caution_border),
      "markdown.alert.important.border" | "md.alert.important.border" => {
        Some(&mut self.markdown_alert_important_border)
      }
      "markdown.alert.note.border" | "md.alert.note.border" => Some(&mut self.markdown_alert_note_border),
      "markdown.alert.tip.border" | "md.alert.tip.border" => Some(&mut self.markdown_alert_tip_border),
      "markdown.alert.warning.border" | "md.alert.warning.border" => Some(&mut self.markdown_alert_warning_border),
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

      "meta.not_set" => Some(&mut self.meta_not_set),
      "meta.value" => Some(&mut self.meta_value),

      "message.created.label" => Some(&mut self.message_created_label),
      "message.success.icon" => Some(&mut self.message_success_icon),
      "message.updated.label" => Some(&mut self.message_updated_label),

      "migrate.count" => Some(&mut self.migrate_count),

      "muted" => Some(&mut self.muted),

      "note.detail.label" => Some(&mut self.note_detail_label),
      "note.detail.separator" => Some(&mut self.note_detail_separator),
      "note.detail.value" => Some(&mut self.note_detail_value),
      "note.list.body" => Some(&mut self.note_list_body),
      "note.list.id" => Some(&mut self.note_list_id),

      "project.list.root" => Some(&mut self.project_list_root),
      "project.show.value" => Some(&mut self.project_show_value),

      "search.expand.separator" => Some(&mut self.search_expand_separator),
      "search.no_results.hint" => Some(&mut self.search_no_results_hint),
      "search.query" => Some(&mut self.search_query),
      "search.summary" => Some(&mut self.search_summary),
      "search.type.label" => Some(&mut self.search_type_label),

      "serve.url" => Some(&mut self.serve_url),

      "status.cancelled" => Some(&mut self.status_cancelled),
      "status.done" => Some(&mut self.status_done),
      "status.in_progress" => Some(&mut self.status_in_progress),
      "status.open" => Some(&mut self.status_open),

      "success" => Some(&mut self.success),
      "tag" => Some(&mut self.tag),
      "tag.list.count" => Some(&mut self.tag_list_count),
      "tag.list.heading" => Some(&mut self.tag_list_heading),

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

/// Return the global theme, falling back to defaults if not yet initialized.
pub fn global() -> &'static Theme {
  THEME.get_or_init(Theme::default)
}

/// Store the resolved theme for global access.
///
/// Must be called before CLI parsing so that `long_about()` and error handlers
/// see user-configured colors. Subsequent calls are no-ops.
pub fn set_global(theme: Theme) {
  THEME.set(theme).ok();
}

/// Returns the [`Palette`] slot that a given token key draws its default color from.
fn palette_for_token(key: &str) -> Option<Palette> {
  use Palette::*;

  match key {
    "artifact.detail.label" => Some(TextMuted),
    "artifact.detail.separator" => Some(Border),
    "artifact.detail.value" => Some(Text),
    "artifact.list.archived.badge" => Some(TextDim),
    "artifact.list.tag.archived" => Some(TextDim),
    "artifact.list.title" => Some(Text),
    "artifact.list.title.archived" => Some(TextDim),

    "banner.author" => Some(Text),
    "banner.author.name" => Some(Accent),
    "banner.gradient.end" => None,
    "banner.gradient.start" => None,
    "banner.shadow" => None,
    "banner.update.command" => Some(Text),
    "banner.update.hint" => Some(TextMuted),
    "banner.update.message" => Some(Warning),
    "banner.update.version" => Some(Warning),
    "banner.version" => Some(Text),
    "banner.version.date" => Some(Primary),
    "banner.version.revision" => Some(Success),

    "border" => Some(Border),

    "config.heading" => Some(Primary),
    "config.label" => Some(TextMuted),
    "config.no_overrides" => Some(TextDim),
    "config.value" => Some(Text),

    "emphasis" => Some(Primary),
    "error" => Some(Error),

    "id.prefix" => Some(Primary),
    "id.rest" => Some(TextMuted),

    "indicator.blocked" => Some(Error),
    "indicator.blocked_by.id" => Some(Primary),
    "indicator.blocked_by.label" => Some(TextMuted),
    "indicator.blocking" => Some(Warning),

    "init.command.prefix" => Some(Border),
    "init.label" => Some(TextMuted),
    "init.section" => Some(TextMuted),
    "init.value" => Some(Text),

    "iteration.detail.count.blocked" => Some(Error),
    "iteration.detail.count.done" => Some(Success),
    "iteration.detail.count.in_progress" => Some(Warning),
    "iteration.detail.count.open" => Some(Text),
    "iteration.detail.label" => Some(TextMuted),
    "iteration.detail.value" => Some(Text),

    "iteration.graph.branch" => Some(Border),
    "iteration.graph.phase.icon" => Some(Primary),
    "iteration.graph.phase.label" => Some(Primary),
    "iteration.graph.phase.name" => Some(TextMuted),
    "iteration.graph.separator" => Some(Border),
    "iteration.graph.title" => Some(Text),

    "iteration.list.summary" => Some(TextMuted),
    "iteration.list.title" => Some(Text),

    "iteration.status.label" => Some(TextMuted),
    "iteration.status.progress" => Some(Primary),
    "iteration.status.value" => Some(Text),

    "list.heading" => Some(Primary),
    "list.summary" => Some(TextMuted),

    "log.debug" => Some(PrimaryLight),
    "log.error" => Some(Error),
    "log.info" => Some(Primary),
    "log.timestamp" => Some(TextDim),
    "log.trace" => Some(TextDim),
    "log.warn" => Some(Warning),

    "markdown.alert.caution.border" => Some(Error),
    "markdown.alert.important.border" => None,
    "markdown.alert.note.border" => Some(Primary),
    "markdown.alert.tip.border" => Some(Success),
    "markdown.alert.warning.border" => Some(Warning),
    "markdown.blockquote" => Some(TextMuted),
    "markdown.blockquote.border" => Some(TextDim),
    "markdown.code.block" => Some(Text),
    "markdown.code.border" => Some(PrimaryDark),
    "markdown.code.inline" => Some(Accent),
    "markdown.emphasis" => None,
    "markdown.heading" => Some(Primary),
    "markdown.link" => Some(Primary),
    "markdown.rule" => Some(Border),
    "markdown.strong" => None,

    "meta.not_set" => Some(TextDim),
    "meta.value" => Some(Text),

    "message.created.label" => Some(Text),
    "message.success.icon" => Some(Success),
    "message.updated.label" => Some(Text),

    "migrate.count" => Some(Primary),

    "muted" => Some(TextMuted),

    "note.detail.label" => Some(TextMuted),
    "note.detail.separator" => Some(Border),
    "note.detail.value" => Some(Text),
    "note.list.body" => Some(TextMuted),
    "note.list.id" => Some(Primary),

    "project.list.root" => Some(Text),
    "project.show.value" => Some(Text),

    "search.expand.separator" => Some(Border),
    "search.no_results.hint" => Some(TextDim),
    "search.query" => Some(Text),
    "search.summary" => Some(TextMuted),
    "search.type.label" => Some(TextMuted),

    "serve.url" => Some(Primary),

    "status.cancelled" => Some(TextDim),
    "status.done" => Some(Success),
    "status.in_progress" => Some(Warning),
    "status.open" => Some(Text),

    "success" => Some(Success),
    "tag" => Some(Primary),
    "tag.list.count" => Some(TextMuted),
    "tag.list.heading" => Some(Primary),

    "task.detail.label" => Some(TextMuted),
    "task.detail.separator" => Some(Border),
    "task.detail.title" => Some(Text),
    "task.detail.value" => Some(Text),

    "task.list.icon.cancelled" => Some(TextDim),
    "task.list.icon.done" => Some(Success),
    "task.list.icon.in_progress" => Some(Warning),
    "task.list.icon.open" => Some(Text),
    "task.list.priority" => Some(TextMuted),
    "task.list.title" => Some(Text),
    "task.list.title.cancelled" => Some(TextDim),

    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod all_tokens {
    use super::*;

    #[test]
    fn it_has_115_token_keys() {
      assert_eq!(ALL_TOKENS.len(), 115);
    }

    #[test]
    fn it_has_a_palette_mapping_for_every_non_inline_token() {
      let none_keys: &[&str] = &[
        "banner.gradient.end",
        "banner.gradient.start",
        "banner.shadow",
        "markdown.alert.important.border",
        "markdown.emphasis",
        "markdown.strong",
      ];
      for key in ALL_TOKENS {
        if none_keys.contains(key) {
          assert_eq!(palette_for_token(key), None, "expected None for {key}");
        } else {
          assert!(palette_for_token(key).is_some(), "expected Some for {key}");
        }
      }
    }

    #[test]
    fn it_has_a_style_mut_entry_for_every_token() {
      let mut theme = Theme::default();

      for key in ALL_TOKENS {
        assert!(theme.style_mut(key).is_some(), "style_mut should handle token `{key}`");
      }
    }
  }

  mod global_fn {
    use super::*;

    #[test]
    fn it_returns_the_same_reference_on_subsequent_calls() {
      let first = global() as *const Theme;
      let second = global() as *const Theme;

      assert_eq!(first, second);
    }
  }

  mod theme_from_config {
    use super::*;

    #[test]
    fn it_builds_without_panic_from_default_settings() {
      let settings = crate::config::Settings::default();

      let _theme = Theme::from_config(&settings);
    }
  }
}
