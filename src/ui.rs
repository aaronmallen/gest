//! Terminal UI layer built on atomic design principles.
//!
//! Atoms are the smallest styled primitives, composites combine atoms into
//! reusable widgets, and views assemble composites into full command outputs.

/// Smallest styled display primitives (badge, icon, label, etc.).
pub mod atoms;
/// Mid-level widgets composed from atoms.
pub mod composites;
/// Row and column layout containers with terminal-aware overflow.
pub mod layout;
/// Markdown-to-styled-terminal renderer.
pub mod markdown;
/// Consolidated theme tokens for the new components system.
pub mod style;
/// Theming subsystem: color constants, palette abstraction, and semantic style tokens.
pub mod theming;
/// Terminal measurement and ANSI-stripping helpers.
pub mod utils;
/// Top-level output renderers, one per CLI command group.
pub mod views;

/// Enable ANSI color output only when stdout is a TTY with color support.
pub fn init() {
  yansi::whenever(yansi::Condition::TTY_AND_COLOR);
}
