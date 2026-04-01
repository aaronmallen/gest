//! Palette color abstraction and token-to-palette mapping.
//!
//! Each [`PaletteColor`] variant represents a semantic color slot.  The
//! [`palette_for_token`] function maps every theme token key to the palette
//! slot it references (or `None` for tokens with inline RGB defaults).

use yansi::Color;

use super::colors;

/// All theme token keys recognized by [`Theme::apply_overrides`].
///
/// This list must stay in sync with the match arms in `apply_overrides`.
/// The test suite verifies completeness.
pub const ALL_TOKEN_KEYS: &[&str] = &[
  "artifact.detail.label",
  "artifact.detail.separator",
  "artifact.detail.value",
  "artifact.list.archived.badge",
  "artifact.list.kind",
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
  "list.heading",
  "list.summary",
  "log.debug",
  "log.error",
  "log.info",
  "log.timestamp",
  "log.trace",
  "log.warn",
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
  "message.created.label",
  "message.success.icon",
  "message.updated.label",
  "muted",
  "search.expand.separator",
  "search.no_results.hint",
  "search.query",
  "search.summary",
  "search.type.label",
  "status.cancelled",
  "status.done",
  "status.in_progress",
  "status.open",
  "success",
  "tag",
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

/// Semantic palette slots that theme tokens can reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteColor {
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

impl PaletteColor {
  /// All palette variants in definition order.
  pub const ALL: [PaletteColor; 11] = [
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
  #[allow(dead_code)]
  pub fn default_color(self) -> Color {
    match self {
      Self::Accent => colors::EMBER,
      Self::Border => colors::BORDER,
      Self::Error => colors::ERROR,
      Self::Primary => colors::AZURE,
      Self::PrimaryDark => colors::AZURE_DARK,
      Self::PrimaryLight => colors::AZURE_LIGHT,
      Self::Success => colors::JADE,
      Self::Text => colors::SILVER,
      Self::TextDim => colors::DIM,
      Self::TextMuted => colors::PEWTER,
      Self::Warning => colors::AMBER,
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

/// Return the palette slot that the given theme token references, or `None`
/// for tokens with inline RGB defaults or no foreground color.
pub fn palette_for_token(key: &str) -> Option<PaletteColor> {
  use PaletteColor::*;

  match key {
    // ── Artifact ──────────────────────────────────────────────
    "artifact.detail.label" => Some(TextMuted),
    "artifact.detail.separator" => Some(Border),
    "artifact.detail.value" => Some(Text),
    "artifact.list.archived.badge" => Some(TextDim),
    "artifact.list.kind" => Some(TextMuted),
    "artifact.list.tag.archived" => Some(TextDim),
    "artifact.list.title" => Some(Text),
    "artifact.list.title.archived" => Some(TextDim),

    // ── Banner ───────────────────────────────────────────────
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

    // ── Border ───────────────────────────────────────────────
    "border" => Some(Border),

    // ── Config ───────────────────────────────────────────────
    "config.heading" => Some(Primary),
    "config.label" => Some(TextMuted),
    "config.no_overrides" => Some(TextDim),
    "config.value" => Some(Text),

    // ── Core ─────────────────────────────────────────────────
    "emphasis" => Some(Primary),
    "error" => Some(Error),

    // ── ID ───────────────────────────────────────────────────
    "id.prefix" => Some(Primary),
    "id.rest" => Some(TextMuted),

    // ── Indicators ───────────────────────────────────────────
    "indicator.blocked" => Some(Error),
    "indicator.blocked_by.id" => Some(Primary),
    "indicator.blocked_by.label" => Some(TextMuted),
    "indicator.blocking" => Some(Warning),

    // ── Init ─────────────────────────────────────────────────
    "init.command.prefix" => Some(Border),
    "init.label" => Some(TextMuted),
    "init.section" => Some(TextMuted),
    "init.value" => Some(Text),

    // ── Iteration detail ─────────────────────────────────────
    "iteration.detail.count.blocked" => Some(Error),
    "iteration.detail.count.done" => Some(Success),
    "iteration.detail.count.in_progress" => Some(Warning),
    "iteration.detail.count.open" => Some(Text),
    "iteration.detail.label" => Some(TextMuted),
    "iteration.detail.value" => Some(Text),

    // ── Iteration graph ──────────────────────────────────────
    "iteration.graph.branch" => Some(Border),
    "iteration.graph.phase.icon" => Some(Primary),
    "iteration.graph.phase.label" => Some(Primary),
    "iteration.graph.phase.name" => Some(TextMuted),
    "iteration.graph.separator" => Some(Border),
    "iteration.graph.title" => Some(Text),

    // ── Iteration list ───────────────────────────────────────
    "iteration.list.summary" => Some(TextMuted),
    "iteration.list.title" => Some(Text),

    // ── List ─────────────────────────────────────────────────
    "list.heading" => Some(Primary),
    "list.summary" => Some(TextMuted),

    // ── Log ──────────────────────────────────────────────────
    "log.debug" => Some(PrimaryLight),
    "log.error" => Some(Error),
    "log.info" => Some(Primary),
    "log.timestamp" => Some(TextDim),
    "log.trace" => Some(TextDim),
    "log.warn" => Some(Warning),

    // ── Markdown ─────────────────────────────────────────────
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

    // ── Messages ─────────────────────────────────────────────
    "message.created.label" => Some(Text),
    "message.success.icon" => Some(Success),
    "message.updated.label" => Some(Text),

    // ── Muted ────────────────────────────────────────────────
    "muted" => Some(TextMuted),

    // ── Search ───────────────────────────────────────────────
    "search.expand.separator" => Some(Border),
    "search.no_results.hint" => Some(TextDim),
    "search.query" => Some(Text),
    "search.summary" => Some(TextMuted),
    "search.type.label" => Some(TextMuted),

    // ── Status ───────────────────────────────────────────────
    "status.cancelled" => Some(TextDim),
    "status.done" => Some(Success),
    "status.in_progress" => Some(Warning),
    "status.open" => Some(Text),

    // ── Success / Tag ────────────────────────────────────────
    "success" => Some(Success),
    "tag" => Some(Primary),

    // ── Task detail ──────────────────────────────────────────
    "task.detail.label" => Some(TextMuted),
    "task.detail.separator" => Some(Border),
    "task.detail.title" => Some(Text),
    "task.detail.value" => Some(Text),

    // ── Task list ────────────────────────────────────────────
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

  #[test]
  fn it_covers_all_token_keys_in_the_mapping() {
    let none_keys: &[&str] = &[
      "banner.gradient.end",
      "banner.gradient.start",
      "banner.shadow",
      "markdown.emphasis",
      "markdown.strong",
    ];
    for key in ALL_TOKEN_KEYS {
      if none_keys.contains(key) {
        assert_eq!(palette_for_token(key), None, "expected None for {key}");
      } else {
        assert!(palette_for_token(key).is_some(), "expected Some for {key}");
      }
    }
  }

  #[test]
  fn it_has_95_token_keys() {
    assert_eq!(ALL_TOKEN_KEYS.len(), 95);
  }

  #[test]
  fn it_has_exactly_11_variants() {
    assert_eq!(PaletteColor::ALL.len(), 11);
  }

  #[test]
  fn it_has_unique_keys_for_all_palette_variants() {
    use std::collections::HashSet;
    let keys: HashSet<&str> = PaletteColor::ALL.iter().map(|p| p.key()).collect();
    assert_eq!(keys.len(), PaletteColor::ALL.len());
  }

  #[test]
  fn it_maps_error_tokens_correctly() {
    assert_eq!(palette_for_token("error"), Some(PaletteColor::Error));
    assert_eq!(palette_for_token("indicator.blocked"), Some(PaletteColor::Error));
    assert_eq!(palette_for_token("log.error"), Some(PaletteColor::Error));
  }

  #[test]
  fn it_maps_every_token_key() {
    for key in ALL_TOKEN_KEYS {
      let _ = palette_for_token(key);
    }
  }

  #[test]
  fn it_maps_primary_tokens_correctly() {
    let primary_tokens = [
      "emphasis",
      "config.heading",
      "id.prefix",
      "list.heading",
      "markdown.heading",
      "markdown.link",
      "tag",
    ];
    for key in primary_tokens {
      assert_eq!(
        palette_for_token(key),
        Some(PaletteColor::Primary),
        "expected Primary for {key}"
      );
    }
  }

  #[test]
  fn it_maps_success_tokens_correctly() {
    assert_eq!(palette_for_token("status.done"), Some(PaletteColor::Success));
    assert_eq!(palette_for_token("success"), Some(PaletteColor::Success));
    assert_eq!(palette_for_token("message.success.icon"), Some(PaletteColor::Success));
  }

  #[test]
  fn it_returns_correct_default_colors() {
    assert_eq!(PaletteColor::Accent.default_color(), colors::EMBER);
    assert_eq!(PaletteColor::Border.default_color(), colors::BORDER);
    assert_eq!(PaletteColor::Error.default_color(), colors::ERROR);
    assert_eq!(PaletteColor::Primary.default_color(), colors::AZURE);
    assert_eq!(PaletteColor::PrimaryDark.default_color(), colors::AZURE_DARK);
    assert_eq!(PaletteColor::PrimaryLight.default_color(), colors::AZURE_LIGHT);
    assert_eq!(PaletteColor::Success.default_color(), colors::JADE);
    assert_eq!(PaletteColor::Text.default_color(), colors::SILVER);
    assert_eq!(PaletteColor::TextDim.default_color(), colors::DIM);
    assert_eq!(PaletteColor::TextMuted.default_color(), colors::PEWTER);
    assert_eq!(PaletteColor::Warning.default_color(), colors::AMBER);
  }

  #[test]
  fn it_returns_none_for_inline_rgb_tokens() {
    assert_eq!(palette_for_token("banner.gradient.start"), None);
    assert_eq!(palette_for_token("banner.gradient.end"), None);
    assert_eq!(palette_for_token("banner.shadow"), None);
  }

  #[test]
  fn it_returns_none_for_modifier_only_tokens() {
    assert_eq!(palette_for_token("markdown.emphasis"), None);
    assert_eq!(palette_for_token("markdown.strong"), None);
  }

  #[test]
  fn it_returns_none_for_unknown_keys() {
    assert_eq!(palette_for_token("nonexistent.token"), None);
  }
}
