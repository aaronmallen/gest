//! Log-related configuration (`[log]` table).

use serde::{Deserialize, Serialize};

use super::env::GEST_LOG__LEVEL;
use crate::logging::LevelFilter;

/// Settings from the `[log]` configuration table.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  /// The configured log level filter.
  level: LevelFilter,
}

impl Settings {
  /// Resolve the effective log level.
  ///
  /// The `GEST_LOG__LEVEL` environment variable takes precedence over the
  /// config-file value. If the env var is unset or unparseable, the
  /// config-file value (or its default) is used.
  pub fn level(&self) -> LevelFilter {
    GEST_LOG__LEVEL.value().ok().unwrap_or(self.level)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod level {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_prefers_env_var_over_config_value() {
      let settings = Settings {
        level: LevelFilter::Warn,
      };

      temp_env::with_var("GEST_LOG__LEVEL", Some("debug"), || {
        assert_eq!(settings.level(), LevelFilter::Debug);
      });
    }

    #[test]
    fn it_uses_config_value_when_env_var_is_unset() {
      let settings = Settings {
        level: LevelFilter::Error,
      };

      temp_env::with_var("GEST_LOG__LEVEL", None::<&str>, || {
        assert_eq!(settings.level(), LevelFilter::Error);
      });
    }

    #[test]
    fn it_falls_back_to_config_value_when_env_var_is_invalid() {
      let settings = Settings {
        level: LevelFilter::Info,
      };

      temp_env::with_var("GEST_LOG__LEVEL", Some("not_a_level"), || {
        assert_eq!(settings.level(), LevelFilter::Info);
      });
    }
  }
}
