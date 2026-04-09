//! Web server configuration (`[serve]` table).

use std::net::{IpAddr, Ipv4Addr};

use getset::CopyGetters;
use serde::{Deserialize, Serialize};

use crate::logging::LevelFilter;

/// Settings from the `[serve]` configuration table.
#[derive(Clone, CopyGetters, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  /// IP address the server should bind to.
  #[getset(get_copy = "pub")]
  bind_address: IpAddr,
  /// Hex-encoded 32-byte HMAC key used to sign CSRF cookies.
  ///
  /// Auto-generated and persisted to the global config file on first startup
  /// when absent. Rotating the key invalidates every outstanding CSRF cookie
  /// on the next request, which will be reissued transparently.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  csrf_signing_key: Option<String>,
  /// File watcher debounce window in milliseconds.
  #[getset(get_copy = "pub")]
  debounce_ms: u64,
  /// Optional log level override applied while `gest serve` is running.
  #[getset(get_copy = "pub")]
  #[serde(default, skip_serializing_if = "Option::is_none")]
  log_level: Option<LevelFilter>,
  /// Whether to automatically open the browser when the server starts.
  #[getset(get_copy = "pub")]
  open: bool,
  /// Port the server should listen on.
  #[getset(get_copy = "pub")]
  port: u16,
}

impl Settings {
  /// Hex-encoded HMAC signing key used by the CSRF middleware.
  pub fn csrf_signing_key(&self) -> Option<&str> {
    self.csrf_signing_key.as_deref()
  }
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      bind_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
      csrf_signing_key: None,
      debounce_ms: 2000,
      log_level: None,
      open: true,
      port: 2300,
    }
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn it_defaults_to_bind_address_localhost() {
    let settings = Settings::default();

    assert_eq!(settings.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
  }

  #[test]
  fn it_defaults_to_debounce_ms_2000() {
    let settings = Settings::default();

    assert_eq!(settings.debounce_ms(), 2000);
  }

  #[test]
  fn it_defaults_to_no_log_level() {
    let settings = Settings::default();

    assert_eq!(settings.log_level(), None);
  }

  #[test]
  fn it_defaults_to_open_true() {
    let settings = Settings::default();

    assert!(settings.open());
  }

  #[test]
  fn it_defaults_to_port_2300() {
    let settings = Settings::default();

    assert_eq!(settings.port(), 2300);
  }

  #[test]
  fn it_deserializes_bind_address() {
    let toml_str = r#"bind_address = "0.0.0.0""#;
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert_eq!(settings.bind_address(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
  }

  #[test]
  fn it_deserializes_debounce_ms() {
    let toml_str = "debounce_ms = 500";
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert_eq!(settings.debounce_ms(), 500);
  }

  #[test]
  fn it_deserializes_log_level() {
    let toml_str = r#"log_level = "debug""#;
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert_eq!(settings.log_level(), Some(LevelFilter::Debug));
  }

  #[test]
  fn it_deserializes_open() {
    let toml_str = "open = false";
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert!(!settings.open());
  }

  #[test]
  fn it_deserializes_port() {
    let toml_str = "port = 8080";
    let settings: Settings = toml::from_str(toml_str).unwrap();

    assert_eq!(settings.port(), 8080);
  }

  #[test]
  fn it_omits_none_log_level_on_serialize() {
    let settings = Settings::default();
    let serialized = toml::to_string(&settings).unwrap();

    assert!(!serialized.contains("log_level"));
  }

  #[test]
  fn it_round_trips_through_toml() {
    let settings = Settings {
      bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
      csrf_signing_key: Some("deadbeef".to_owned()),
      debounce_ms: 500,
      log_level: Some(LevelFilter::Debug),
      open: false,
      port: 9090,
    };
    let serialized = toml::to_string(&settings).unwrap();
    let deserialized: Settings = toml::from_str(&serialized).unwrap();

    assert_eq!(settings, deserialized);
  }
}
