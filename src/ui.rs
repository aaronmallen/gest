//! UI rendering and style infrastructure.

/// Reusable UI components for terminal output.
pub mod components;
pub mod json;
/// Markdown-to-styled-terminal renderer.
pub mod markdown;
/// Semantic style tokens and theme resolution.
pub mod style;

use yansi::Condition;

/// Initialize the UI subsystem by enabling colored output when stdout is a TTY.
pub fn init() {
  yansi::whenever(Condition::TTY_AND_COLOR);
}
