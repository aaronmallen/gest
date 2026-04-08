//! Pager-related configuration (`[pager]` table).

use getset::CopyGetters;
use serde::{Deserialize, Serialize};

/// Settings from the `[pager]` configuration table.
///
/// Controls whether and how gest pipes long output through a pager program.
/// Wiring into the pager helper is intentionally deferred -- this struct only
/// defines the schema, defaults, and serde plumbing so the values can be read
/// and written via `gest config get` / `gest config set`.
#[derive(Clone, CopyGetters, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  /// Optional pager command override (e.g. `"less -FR"`).
  ///
  /// An empty string is treated as "unset" by [`Settings::command`], so users
  /// can disable an inherited value from a less-specific config file by
  /// writing `pager.command = ""`. This mirrors the convention used by the
  /// `PAGER` environment variable.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  command: Option<String>,
  /// Whether the pager is enabled.
  ///
  /// Defaults to `true`. When `false`, gest should print long output directly
  /// to stdout without spawning a pager process.
  #[getset(get_copy = "pub")]
  enabled: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      command: None,
      enabled: true,
    }
  }
}

impl Settings {
  /// The configured pager command, if any.
  ///
  /// Returns `None` when the value is unset _or_ set to an empty string. The
  /// empty-string case lets users explicitly clear an inherited value via
  /// `gest config set pager.command ""`.
  pub fn command(&self) -> Option<&str> {
    self.command.as_deref().filter(|value| !value.is_empty())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod default {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_defaults_command_to_none() {
      let settings = Settings::default();

      assert_eq!(settings.command(), None);
    }

    #[test]
    fn it_defaults_enabled_to_true() {
      let settings = Settings::default();

      assert!(settings.enabled());
    }
  }

  mod command {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_none_for_an_empty_string() {
      let settings = Settings {
        command: Some(String::new()),
        enabled: true,
      };

      assert_eq!(settings.command(), None);
    }

    #[test]
    fn it_returns_some_for_a_non_empty_value() {
      let settings = Settings {
        command: Some("less -FR".to_string()),
        enabled: true,
      };

      assert_eq!(settings.command(), Some("less -FR"));
    }
  }

  mod toml_round_trip {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_deserializes_command() {
      let toml_str = r#"command = "less -FR""#;
      let settings: Settings = toml::from_str(toml_str).unwrap();

      assert_eq!(settings.command(), Some("less -FR"));
    }

    #[test]
    fn it_deserializes_empty_command_as_disabled() {
      let toml_str = r#"command = """#;
      let settings: Settings = toml::from_str(toml_str).unwrap();

      assert_eq!(settings.command(), None);
    }

    #[test]
    fn it_deserializes_enabled() {
      let toml_str = "enabled = false";
      let settings: Settings = toml::from_str(toml_str).unwrap();

      assert!(!settings.enabled());
    }

    #[test]
    fn it_omits_none_command_on_serialize() {
      let settings = Settings::default();
      let serialized = toml::to_string(&settings).unwrap();

      assert!(!serialized.contains("command"));
    }

    #[test]
    fn it_round_trips_through_toml() {
      let settings = Settings {
        command: Some("less -FR".to_string()),
        enabled: false,
      };
      let serialized = toml::to_string(&settings).unwrap();
      let deserialized: Settings = toml::from_str(&serialized).unwrap();

      assert_eq!(settings, deserialized);
    }

    #[test]
    fn it_uses_defaults_when_table_is_empty() {
      let settings: Settings = toml::from_str("").unwrap();

      assert!(settings.enabled());
      assert_eq!(settings.command(), None);
    }
  }
}
