//! Pipe long command output through the user's preferred pager.
//!
//! The public entry point is [`page`], which decides whether to spawn a pager
//! or print directly to stdout based on a fixed policy:
//!
//! 1. Stdout is not a TTY → print directly
//! 2. The user passed `--no-pager` → print directly
//! 3. `pager.enabled = false` in config → print directly
//! 4. The resolved pager command is empty → print directly
//! 5. The content fits on screen (`lines ≤ terminal_rows - 1`) → print directly
//! 6. Otherwise spawn the pager and pipe content to its stdin
//!
//! On any spawn or terminal-size failure the helper falls back to a direct
//! print so output is never lost.

use std::{
  io::{IsTerminal, Result, Write},
  process::{Command, Stdio},
};

use crate::AppContext;

/// Default pager invocation when neither config nor environment specifies one.
///
/// `less -R` passes ANSI escapes through unchanged so colored output renders
/// correctly when paged.
const DEFAULT_PAGER: &str = "less -R";

/// Pipe `content` through the user's preferred pager when the policy allows it.
///
/// See the module docs for the full decision order. The function never errors
/// on a missing or broken pager: any spawn or write failure falls back to
/// printing the content directly to stdout.
pub fn page(content: &str, context: &AppContext) -> Result<()> {
  let is_tty = std::io::stdout().is_terminal();
  let line_count = content.lines().count();
  let terminal_rows = terminal_size::terminal_size().map(|(_, h)| h.0);

  let policy_allows = should_page(
    line_count,
    *context.no_pager(),
    context.settings().pager().enabled(),
    is_tty,
    terminal_rows,
  );

  if !policy_allows {
    print!("{content}");
    return Ok(());
  }

  let pager_cmd = resolve_pager(context.settings().pager().command());
  if pager_cmd.is_empty() {
    print!("{content}");
    return Ok(());
  }

  let parts = match shell_words::split(&pager_cmd) {
    Ok(parts) => parts,
    Err(_) => {
      print!("{content}");
      return Ok(());
    }
  };

  let (program, args) = match parts.split_first() {
    Some(pair) => pair,
    None => {
      print!("{content}");
      return Ok(());
    }
  };

  let mut child = match Command::new(program).args(args).stdin(Stdio::piped()).spawn() {
    Ok(child) => child,
    Err(_) => {
      print!("{content}");
      return Ok(());
    }
  };

  if let Some(mut stdin) = child.stdin.take() {
    // Ignore broken-pipe errors (user quit the pager early).
    let _ = stdin.write_all(content.as_bytes());
  }

  child.wait()?;
  Ok(())
}

/// Resolve the pager command using the documented precedence.
///
/// Order, highest to lowest:
///
/// 1. `config_command` (the `pager.command` config value) when non-empty
/// 2. `$GEST_PAGER` when non-empty
/// 3. `$PAGER` when non-empty
/// 4. The hard-coded [`DEFAULT_PAGER`]
fn resolve_pager(config_command: Option<&str>) -> String {
  if let Some(command) = config_command.filter(|value| !value.is_empty()) {
    return command.to_string();
  }

  if let Ok(value) = std::env::var("GEST_PAGER")
    && !value.is_empty()
  {
    return value;
  }

  if let Ok(value) = std::env::var("PAGER")
    && !value.is_empty()
  {
    return value;
  }

  DEFAULT_PAGER.to_string()
}

/// Decide whether the configured policy allows spawning a pager.
///
/// `terminal_rows` is `None` when [`terminal_size`] could not detect the
/// terminal height; in that case the height check is skipped (i.e. assumed to
/// require paging) so that interactive output is not silently swallowed.
fn should_page(line_count: usize, no_pager: bool, enabled: bool, is_tty: bool, terminal_rows: Option<u16>) -> bool {
  if !is_tty {
    return false;
  }

  if no_pager {
    return false;
  }

  if !enabled {
    return false;
  }

  match terminal_rows {
    Some(rows) => line_count > (rows as usize).saturating_sub(1),
    None => true,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod page {
    use super::*;
    use crate::{AppContext, config::Settings, store};

    #[tokio::test]
    async fn it_returns_ok_in_test_environment() {
      let (store, _tmp) = store::open_temp().await.unwrap();
      let context = AppContext {
        gest_dir: None,
        no_pager: false,
        project_id: None,
        settings: Settings::default(),
        store,
      };

      let result = page("hello world\n", &context);

      assert!(result.is_ok());
    }
  }

  mod resolve_pager {
    use temp_env::with_vars;

    use super::*;

    #[test]
    fn it_defaults_to_less_dash_r() {
      with_vars([("GEST_PAGER", None::<&str>), ("PAGER", None::<&str>)], || {
        assert_eq!(resolve_pager(None), "less -R");
      });
    }

    #[test]
    fn it_ignores_empty_config_command() {
      with_vars([("GEST_PAGER", None::<&str>), ("PAGER", None::<&str>)], || {
        assert_eq!(resolve_pager(Some("")), "less -R");
      });
    }

    #[test]
    fn it_ignores_empty_env_values() {
      with_vars([("GEST_PAGER", Some("")), ("PAGER", Some(""))], || {
        assert_eq!(resolve_pager(None), "less -R");
      });
    }

    #[test]
    fn it_prefers_config_over_env() {
      with_vars([("GEST_PAGER", Some("most")), ("PAGER", Some("more"))], || {
        assert_eq!(resolve_pager(Some("less -FR")), "less -FR");
      });
    }

    #[test]
    fn it_prefers_gest_pager_over_pager() {
      with_vars([("GEST_PAGER", Some("most")), ("PAGER", Some("more"))], || {
        assert_eq!(resolve_pager(None), "most");
      });
    }

    #[test]
    fn it_respects_gest_pager_env() {
      with_vars([("GEST_PAGER", Some("most")), ("PAGER", None::<&str>)], || {
        assert_eq!(resolve_pager(None), "most");
      });
    }

    #[test]
    fn it_respects_pager_env() {
      with_vars([("GEST_PAGER", None::<&str>), ("PAGER", Some("more"))], || {
        assert_eq!(resolve_pager(None), "more");
      });
    }
  }

  mod should_page {
    use super::*;

    #[test]
    fn it_pages_when_content_exceeds_terminal_rows() {
      assert!(should_page(50, false, true, true, Some(24)));
    }

    #[test]
    fn it_pages_when_terminal_size_is_unknown() {
      assert!(should_page(1, false, true, true, None));
    }

    #[test]
    fn it_skips_paging_when_content_fits_on_screen() {
      assert!(!should_page(10, false, true, true, Some(24)));
    }

    #[test]
    fn it_skips_paging_when_content_is_one_short_of_terminal_height() {
      // 23 lines on a 24-row terminal: lines == rows - 1, so we still skip.
      assert!(!should_page(23, false, true, true, Some(24)));
    }

    #[test]
    fn it_skips_paging_when_no_pager_flag_is_set() {
      assert!(!should_page(1000, true, true, true, Some(24)));
    }

    #[test]
    fn it_skips_paging_when_pager_is_disabled_in_config() {
      assert!(!should_page(1000, false, false, true, Some(24)));
    }

    #[test]
    fn it_skips_paging_when_stdout_is_not_a_tty() {
      assert!(!should_page(1000, false, true, false, Some(24)));
    }
  }
}
