//! UI rendering and style infrastructure.

/// Reusable UI components for terminal output.
pub mod components;
/// Shared output-mode (`--json`/`--quiet`/`--raw`) flags for command structs.
pub mod json;
/// Markdown-to-styled-terminal renderer.
pub mod markdown;
/// Semantic style tokens and theme resolution.
pub mod style;

use yansi::Condition;

/// Initialize the UI subsystem by enabling colored output when stdout is a TTY
/// or when `CLICOLOR_FORCE` is set (a common convention honored by many CLIs).
pub fn init() {
  // `CLICOLOR_FORCE=1` forces color output on even when stdout is not a TTY,
  // which is how integration tests observe styled prefixes.
  if std::env::var_os("CLICOLOR_FORCE").map(|v| v != "0").unwrap_or(false) {
    yansi::enable();
    return;
  }
  yansi::whenever(Condition::TTY_AND_COLOR);
}
