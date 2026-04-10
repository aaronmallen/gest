//! Shared interactive prompts for CLI subcommands.
//!
//! These helpers centralize the wording and input parsing used by destructive
//! commands so that behavior (and scriptable `--yes` bypass semantics) stay
//! consistent across subcommands.

use super::Error;
use crate::ui::components::ConfirmPrompt;

/// Prompt the user to confirm a destructive action against `target`.
///
/// When `yes` is `true` the prompt is skipped and `Ok(true)` is returned,
/// matching the scriptable `--yes` / `--force` convention used across the CLI.
/// Otherwise an interactive Yes/No selector is rendered using crossterm for
/// raw terminal input. The default selection is No.
pub fn confirm_destructive(action: &str, target: &str, yes: bool) -> Result<bool, Error> {
  if yes {
    return Ok(true);
  }

  let description = format!("About to {action} {target}. Continue?");
  let confirmed = ConfirmPrompt::new(description).confirm()?;
  Ok(confirmed)
}

#[cfg(test)]
mod tests {
  use super::*;

  mod confirm_destructive {
    use super::*;

    #[test]
    fn it_bypasses_prompt_when_yes_is_true() {
      let confirmed = confirm_destructive("delete", "task abc", true).unwrap();

      assert!(confirmed);
    }
  }
}
