//! Web server configuration settings.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

/// Default bind address (localhost only).
const DEFAULT_BIND_ADDRESS: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

/// Default port for the built-in web server.
const DEFAULT_PORT: u16 = 2300;

/// Configuration for the `[serve]` section.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
  bind_address: IpAddr,
  open: bool,
  port: u16,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      bind_address: DEFAULT_BIND_ADDRESS,
      open: true,
      port: DEFAULT_PORT,
    }
  }
}

impl Settings {
  /// The IP address the server should bind to.
  pub fn bind_address(&self) -> IpAddr {
    self.bind_address
  }

  /// Whether to automatically open the browser when the server starts.
  pub fn open(&self) -> bool {
    self.open
  }

  /// The port the server should listen on.
  pub fn port(&self) -> u16 {
    self.port
  }
}

#[cfg(test)]
mod tests {
  use std::net::Ipv4Addr;

  use pretty_assertions::assert_eq;

  use super::*;

  #[test]
  fn it_defaults_to_bind_address_localhost() {
    let settings = Settings::default();

    assert_eq!(settings.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
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
  fn it_round_trips_through_toml() {
    let settings = Settings {
      bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
      open: false,
      port: 9090,
    };
    let serialized = toml::to_string(&settings).unwrap();
    let deserialized: Settings = toml::from_str(&serialized).unwrap();

    assert_eq!(settings, deserialized);
  }
}
