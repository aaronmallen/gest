//! Shared interactive prompts for CLI subcommands.
//!
//! These helpers centralize the wording and input parsing used by destructive
//! commands so that behavior (and scriptable `--yes` bypass semantics) stay
//! consistent across subcommands.

#![allow(dead_code)]

use std::io::{self, BufRead, Write};

use super::Error;

/// Prompt the user to confirm a destructive action against `target`.
///
/// When `yes` is `true` the prompt is skipped and `Ok(true)` is returned,
/// matching the scriptable `--yes` / `--force` convention used across the CLI.
/// Otherwise the helper writes `About to <action> <target>. Continue? [y/N] `
/// to stdout and reads a single line from stdin. The input is matched
/// case-insensitively against `y` and `yes`; any other response (including
/// EOF or an empty line) is treated as a decline.
pub fn confirm_destructive(action: &str, target: &str, yes: bool) -> Result<bool, Error> {
  confirm_destructive_with(action, target, yes, &mut io::stdout(), &mut io::stdin().lock())
}

/// Core implementation of [`confirm_destructive`] with injectable IO for tests.
fn confirm_destructive_with<W: Write, R: BufRead>(
  action: &str,
  target: &str,
  yes: bool,
  writer: &mut W,
  reader: &mut R,
) -> Result<bool, Error> {
  if yes {
    return Ok(true);
  }

  write!(writer, "About to {action} {target}. Continue? [y/N] ")?;
  writer.flush()?;

  let mut line = String::new();
  if reader.read_line(&mut line)? == 0 {
    return Ok(false);
  }

  let answer = line.trim().to_ascii_lowercase();
  Ok(matches!(answer.as_str(), "y" | "yes"))
}

#[cfg(test)]
mod tests {
  use super::*;

  mod confirm_destructive_with {
    use super::*;

    fn run(input: &str, yes: bool) -> (bool, String) {
      let mut writer: Vec<u8> = Vec::new();
      let mut reader = io::Cursor::new(input.as_bytes().to_vec());
      let result = confirm_destructive_with("delete", "task abc", yes, &mut writer, &mut reader).unwrap();

      (result, String::from_utf8(writer).unwrap())
    }

    #[test]
    fn it_accepts_uppercase_y() {
      let (confirmed, _) = run("Y\n", false);

      assert!(confirmed);
    }

    #[test]
    fn it_accepts_lowercase_y() {
      let (confirmed, _) = run("y\n", false);

      assert!(confirmed);
    }

    #[test]
    fn it_accepts_mixed_case_yes() {
      let (confirmed, _) = run("YeS\n", false);

      assert!(confirmed);
    }

    #[test]
    fn it_bypasses_prompt_when_yes_is_true() {
      let (confirmed, written) = run("", true);

      assert!(confirmed);
      assert!(written.is_empty());
    }

    #[test]
    fn it_declines_on_empty_line() {
      let (confirmed, _) = run("\n", false);

      assert!(!confirmed);
    }

    #[test]
    fn it_declines_on_eof() {
      let (confirmed, _) = run("", false);

      assert!(!confirmed);
    }

    #[test]
    fn it_declines_on_other_input() {
      let (confirmed, _) = run("maybe\n", false);

      assert!(!confirmed);
    }

    #[test]
    fn it_writes_action_and_target_in_prompt() {
      let (_, written) = run("n\n", false);

      assert!(written.contains("About to delete task abc. Continue? [y/N]"));
    }
  }
}
