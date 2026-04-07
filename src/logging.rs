//! Logging infrastructure for gest.
//!
//! Provides a custom [`Logger`] that writes themed, level-prefixed messages to
//! stderr, and a serializable [`LevelFilter`] enum that integrates with both
//! the config file (`[log]` table) and the `GEST_LOG__LEVEL` environment variable.

use std::{borrow::Cow, io::Write};

use log::{Level, Log, Metadata, Record};
use serde::{Deserialize, Serialize};
use typed_env::{EnvarError, EnvarParse, EnvarParser, ErrorReason};
use yansi::Paint;

/// Log level filter mirroring [`log::LevelFilter`] with serde and env-var support.
///
/// Defaults to [`Warn`](Self::Warn). Accepts both named strings (`"debug"`,
/// `"info"`, …) and numeric levels (`0`–`5`) when parsed from environment
/// variables or config files.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LevelFilter {
  Debug,
  Error,
  Info,
  Off,
  Trace,
  #[default]
  Warn,
}

/// Parse a [`LevelFilter`] from an environment variable string.
///
/// Accepts case-insensitive level names (`debug`, `INFO`, …) or numeric
/// values where `0` = Off and `5` = Trace.
impl EnvarParse<LevelFilter> for EnvarParser<LevelFilter> {
  fn parse(varname: Cow<'static, str>, value: &str) -> Result<LevelFilter, EnvarError> {
    match value.trim().to_ascii_lowercase().as_ref() {
      "debug" | "4" => Ok(LevelFilter::Debug),
      "error" | "1" => Ok(LevelFilter::Error),
      "info" | "3" => Ok(LevelFilter::Info),
      "off" | "0" => Ok(LevelFilter::Off),
      "trace" | "5" => Ok(LevelFilter::Trace),
      "warn" | "2" => Ok(LevelFilter::Warn),
      _ => {
        let value = value.to_string();
        Err(EnvarError::ParseError {
          varname,
          typename: std::any::type_name::<LevelFilter>(),
          value: value.clone(),
          reason: ErrorReason::new(move || format!("invalid log level: {}", value)),
        })
      }
    }
  }
}

/// Convert to the [`log`] crate's native [`LevelFilter`](log::LevelFilter).
impl From<LevelFilter> for log::LevelFilter {
  fn from(level: LevelFilter) -> Self {
    match level {
      LevelFilter::Debug => log::LevelFilter::Debug,
      LevelFilter::Error => log::LevelFilter::Error,
      LevelFilter::Info => log::LevelFilter::Info,
      LevelFilter::Off => log::LevelFilter::Off,
      LevelFilter::Trace => log::LevelFilter::Trace,
      LevelFilter::Warn => log::LevelFilter::Warn,
    }
  }
}

/// A minimal [`Log`] implementation that writes themed, level-prefixed lines to stderr.
///
/// Level prefixes are colored using the active [`Theme`](crate::ui::style::Theme) so
/// they respect user palette and token overrides.
pub struct Logger;

impl Log for Logger {
  fn enabled(&self, _metadata: &Metadata) -> bool {
    true
  }

  fn flush(&self) {}

  /// Format and write a single log record to stderr.
  ///
  /// Each line is prefixed with a fixed-width, theme-colored level tag
  /// (e.g. `DEBUG`, `WARN `). Output failures are silently ignored to
  /// avoid recursive logging.
  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    let level = record.level();
    let theme = crate::ui::style::global();
    let prefix = match level {
      Level::Debug => "DEBUG".paint(*theme.log_debug()),
      Level::Error => "ERROR".paint(*theme.log_error()),
      Level::Info => "INFO ".paint(*theme.log_info()),
      Level::Trace => "TRACE".paint(*theme.log_trace()),
      Level::Warn => "WARN ".paint(*theme.log_warn()),
    };

    let _ = writeln!(std::io::stderr(), "{prefix} {}", record.args());
  }
}

/// Install the global [`Logger`] and set the initial max log level.
///
/// The level may be adjusted later (e.g. after config is loaded) via
/// [`log::set_max_level`]. Repeated calls to [`log::set_logger`] are
/// harmless — only the first succeeds.
pub fn init(level: impl Into<log::LevelFilter>, _theme: &crate::ui::theming::theme::Theme) {
  let _ = log::set_logger(&Logger);
  log::set_max_level(level.into());
}

/// Legacy compatibility: install the logger with a numeric level before
/// the user's theme is loaded. Removed once the CLI is fully rewritten.
pub fn init_early(level: log::LevelFilter) {
  let _ = log::set_logger(&Logger);
  log::set_max_level(level);
}

/// Legacy compatibility: resolve a [`log::LevelFilter`] from a verbosity
/// count, an optional environment override, and an optional config override.
///
/// Resolution precedence: env > config > verbosity-derived default.
pub fn resolve_level(verbosity: u8, env_level: Option<&str>, config_level: Option<&str>) -> log::LevelFilter {
  if let Some(env) = env_level
    && let Ok(parsed) = env.parse::<LevelFilter>()
  {
    return parsed.into();
  }
  if let Some(cfg) = config_level
    && let Ok(parsed) = cfg.parse::<LevelFilter>()
  {
    return parsed.into();
  }
  match verbosity {
    0 => log::LevelFilter::Warn,
    1 => log::LevelFilter::Info,
    2 => log::LevelFilter::Debug,
    _ => log::LevelFilter::Trace,
  }
}

/// Legacy compatibility: same as [`resolve_level`] but with an explicit default.
pub fn resolve_level_with_default(
  verbosity: u8,
  env_level: Option<&str>,
  config_level: Option<&str>,
  default: log::LevelFilter,
) -> log::LevelFilter {
  if let Some(env) = env_level
    && let Ok(parsed) = env.parse::<LevelFilter>()
  {
    return parsed.into();
  }
  if let Some(cfg) = config_level
    && let Ok(parsed) = cfg.parse::<LevelFilter>()
  {
    return parsed.into();
  }
  match verbosity {
    0 => default,
    1 => log::LevelFilter::Info,
    2 => log::LevelFilter::Debug,
    _ => log::LevelFilter::Trace,
  }
}

impl std::str::FromStr for LevelFilter {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim().to_ascii_lowercase().as_str() {
      "debug" | "4" => Ok(Self::Debug),
      "error" | "1" => Ok(Self::Error),
      "info" | "3" => Ok(Self::Info),
      "off" | "0" => Ok(Self::Off),
      "trace" | "5" => Ok(Self::Trace),
      "warn" | "2" => Ok(Self::Warn),
      other => Err(format!("invalid log level: {other}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod level_filter_from {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_converts_all_variants_to_log_level_filter() {
      let cases = [
        (LevelFilter::Debug, log::LevelFilter::Debug),
        (LevelFilter::Error, log::LevelFilter::Error),
        (LevelFilter::Info, log::LevelFilter::Info),
        (LevelFilter::Off, log::LevelFilter::Off),
        (LevelFilter::Trace, log::LevelFilter::Trace),
        (LevelFilter::Warn, log::LevelFilter::Warn),
      ];

      for (input, expected) in cases {
        assert_eq!(log::LevelFilter::from(input), expected, "failed for {input:?}");
      }
    }
  }

  mod level_filter_parse {
    use std::borrow::Cow;

    use pretty_assertions::assert_eq;
    use typed_env::{EnvarParse, EnvarParser};

    use super::*;

    fn parse(value: &str) -> Result<LevelFilter, typed_env::EnvarError> {
      EnvarParser::<LevelFilter>::parse(Cow::Borrowed("TEST_VAR"), value)
    }

    #[test]
    fn it_parses_all_named_levels() {
      let cases = [
        ("debug", LevelFilter::Debug),
        ("error", LevelFilter::Error),
        ("info", LevelFilter::Info),
        ("off", LevelFilter::Off),
        ("trace", LevelFilter::Trace),
        ("warn", LevelFilter::Warn),
      ];

      for (input, expected) in cases {
        assert_eq!(parse(input).unwrap(), expected, "failed for {input:?}");
      }
    }

    #[test]
    fn it_parses_all_numeric_levels() {
      let cases = [
        ("0", LevelFilter::Off),
        ("1", LevelFilter::Error),
        ("2", LevelFilter::Warn),
        ("3", LevelFilter::Info),
        ("4", LevelFilter::Debug),
        ("5", LevelFilter::Trace),
      ];

      for (input, expected) in cases {
        assert_eq!(parse(input).unwrap(), expected, "failed for {input:?}");
      }
    }

    #[test]
    fn it_is_case_insensitive() {
      assert_eq!(parse("DEBUG").unwrap(), LevelFilter::Debug);
      assert_eq!(parse("Info").unwrap(), LevelFilter::Info);
      assert_eq!(parse("WARN").unwrap(), LevelFilter::Warn);
    }

    #[test]
    fn it_trims_whitespace() {
      assert_eq!(parse("  debug  ").unwrap(), LevelFilter::Debug);
      assert_eq!(parse("\ttrace\n").unwrap(), LevelFilter::Trace);
    }

    #[test]
    fn it_rejects_invalid_values() {
      assert!(parse("invalid").is_err());
      assert!(parse("6").is_err());
      assert!(parse("").is_err());
    }
  }

  mod level_filter_serde {
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Wrapper {
      level: LevelFilter,
    }

    #[test]
    fn it_roundtrips_all_variants() {
      let variants = [
        LevelFilter::Debug,
        LevelFilter::Error,
        LevelFilter::Info,
        LevelFilter::Off,
        LevelFilter::Trace,
        LevelFilter::Warn,
      ];

      for variant in variants {
        let wrapper = Wrapper {
          level: variant,
        };
        let serialized = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized, wrapper, "roundtrip failed for {variant:?}");
      }
    }

    #[test]
    fn it_deserializes_lowercase_names() {
      let toml_str = "level = \"debug\"";
      let wrapper: Wrapper = toml::from_str(toml_str).unwrap();

      assert_eq!(wrapper.level, LevelFilter::Debug);
    }
  }
}
