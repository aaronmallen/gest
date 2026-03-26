use std::{io::Write, sync::OnceLock};

use log::{Level, Log, Metadata, Record};
use yansi::{Paint, Style};

use crate::ui::theme::Theme;

static LOG_STYLES: OnceLock<LogStyles> = OnceLock::new();
static LOGGER: Logger = Logger;

struct Logger;

impl Log for Logger {
  fn enabled(&self, _metadata: &Metadata) -> bool {
    true
  }

  fn flush(&self) {}

  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    let level = record.level();
    let default_style = Style::new();
    let styles = LOG_STYLES.get();
    let prefix = match level {
      Level::Debug => "DEBUG".paint(styles.map_or(default_style, |s| s.debug)),
      Level::Error => "ERROR".paint(styles.map_or(default_style, |s| s.error)),
      Level::Info => " INFO".paint(styles.map_or(default_style, |s| s.info)),
      Level::Trace => "TRACE".paint(styles.map_or(default_style, |s| s.trace)),
      Level::Warn => " WARN".paint(styles.map_or(default_style, |s| s.warn)),
    };

    let _ = writeln!(std::io::stderr(), "{prefix} {}", record.args());
  }
}

struct LogStyles {
  debug: Style,
  error: Style,
  info: Style,
  trace: Style,
  warn: Style,
}

impl From<&Theme> for LogStyles {
  fn from(theme: &Theme) -> Self {
    Self {
      debug: theme.log_debug,
      error: theme.log_error,
      info: theme.log_info,
      trace: theme.log_trace,
      warn: theme.log_warn,
    }
  }
}

/// Register the global logger and set the initial level.
///
/// Call this early -- before config loading -- so that discovery log calls
/// are captured.  Styles are *not* set here because the theme depends on
/// config which hasn't been loaded yet.
pub fn init_early(level: log::LevelFilter) {
  let _ = log::set_logger(&LOGGER);
  log::set_max_level(level);
}

/// Apply theme styles and (optionally) adjust the log level.
///
/// Call this after config is available so log output is properly styled and
/// the level reflects the full precedence chain (CLI > env > config > default).
pub fn init(level: log::LevelFilter, theme: &Theme) {
  let _ = LOG_STYLES.set(LogStyles::from(theme));
  log::set_max_level(level);
}

pub fn resolve_level(verbosity: u8, env_level: Option<&str>, config_level: Option<&str>) -> log::LevelFilter {
  if verbosity > 0 {
    return match verbosity {
      1 => log::LevelFilter::Info,
      2 => log::LevelFilter::Debug,
      _ => log::LevelFilter::Trace,
    };
  }

  if let Some(level) = env_level.and_then(parse_level) {
    return level;
  }

  if let Some(level) = config_level.and_then(parse_level) {
    return level;
  }

  log::LevelFilter::Warn
}

fn parse_level(s: &str) -> Option<log::LevelFilter> {
  match s.to_ascii_lowercase().as_str() {
    "error" => Some(log::LevelFilter::Error),
    "warn" => Some(log::LevelFilter::Warn),
    "info" => Some(log::LevelFilter::Info),
    "debug" => Some(log::LevelFilter::Debug),
    "trace" => Some(log::LevelFilter::Trace),
    "off" => Some(log::LevelFilter::Off),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod resolve_level {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_clamps_beyond_vvv_to_trace() {
      assert_eq!(resolve_level(5, None, None), log::LevelFilter::Trace);
    }

    #[test]
    fn it_defaults_to_warn() {
      assert_eq!(resolve_level(0, None, None), log::LevelFilter::Warn);
    }

    #[test]
    fn it_falls_back_to_config() {
      assert_eq!(resolve_level(0, None, Some("trace")), log::LevelFilter::Trace);
    }

    #[test]
    fn it_ignores_invalid_config_value() {
      assert_eq!(resolve_level(0, None, Some("invalid")), log::LevelFilter::Warn);
    }

    #[test]
    fn it_ignores_invalid_env_value() {
      assert_eq!(resolve_level(0, Some("invalid"), None), log::LevelFilter::Warn);
    }

    #[test]
    fn it_is_case_insensitive() {
      assert_eq!(resolve_level(0, Some("DEBUG"), None), log::LevelFilter::Debug);
    }

    #[test]
    fn it_maps_v_to_info() {
      assert_eq!(resolve_level(1, None, None), log::LevelFilter::Info);
    }

    #[test]
    fn it_maps_vv_to_debug() {
      assert_eq!(resolve_level(2, None, None), log::LevelFilter::Debug);
    }

    #[test]
    fn it_maps_vvv_to_trace() {
      assert_eq!(resolve_level(3, None, None), log::LevelFilter::Trace);
    }

    #[test]
    fn it_prefers_cli_over_env() {
      assert_eq!(resolve_level(1, Some("debug"), None), log::LevelFilter::Info);
    }

    #[test]
    fn it_prefers_env_over_config() {
      assert_eq!(resolve_level(0, Some("debug"), Some("trace")), log::LevelFilter::Debug);
    }
  }
}
