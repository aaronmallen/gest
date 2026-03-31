//! Logging configuration settings.

use serde::{Deserialize, Serialize};

/// Configuration for the `[log]` section.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  level: Option<String>,
}

impl Settings {
  /// Returns the configured log level filter string, if any.
  pub fn level(&self) -> Option<&str> {
    self.level.as_deref()
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn it_defaults_to_no_level() {
    let settings = Settings::default();

    assert_eq!(settings.level(), None);
  }

  #[test]
  fn it_deserializes_level() {
    let toml_str = r#"level = "debug""#;
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert_eq!(settings.level(), Some("debug"));
  }

  #[test]
  fn it_omits_none_level_on_serialize() {
    let settings = Settings::default();
    let serialized = toml::to_string(&settings).unwrap();

    assert!(!serialized.contains("level"));
  }
}
