use std::{
  fs,
  net::{IpAddr, SocketAddr},
  path::{Path, PathBuf},
};

use clap::Args;
use toml::{Table, Value};

use crate::{
  AppContext,
  cli::Error,
  config::{self, env::GEST_CONFIG},
  web::CsrfKey,
};

/// Start the web dashboard server.
#[derive(Args, Debug)]
pub struct Command {
  /// Address to bind to (overrides `[serve].bind_address`).
  #[arg(long = "bind", short = 'b', alias = "host")]
  bind_address: Option<IpAddr>,
  /// File watcher debounce in milliseconds (overrides `[serve].debounce_ms`).
  #[arg(long)]
  debounce_ms: Option<u64>,
  /// Suppress automatic browser opening.
  #[arg(long)]
  no_open: bool,
  /// Port to listen on (overrides `[serve].port`).
  #[arg(long, short)]
  port: Option<u16>,
}

impl Command {
  /// Start the embedded web dashboard, binding to the resolved address and port.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    log::debug!("serve: entry");
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let serve_config = context.settings().serve();

    let bind_address = self.bind_address.unwrap_or_else(|| serve_config.bind_address());
    let port = self.port.unwrap_or_else(|| serve_config.port());
    let debounce_ms = self.debounce_ms.unwrap_or_else(|| serve_config.debounce_ms());
    let open = !self.no_open && serve_config.open();

    if let Some(level) = serve_config.log_level() {
      log::set_max_level(level.into());
    }

    let addr = SocketAddr::from((bind_address, port));
    let url = format!("http://{addr}");
    println!("  starting gest dashboard at {url}");

    if open && let Err(e) = open::that(&url) {
      log::warn!("failed to open browser: {e}");
    }

    let data_dir = context.settings().storage().data_dir()?;
    let cache_dir = context.settings().storage().cache_dir()?;
    let socket_path = crate::web::reload_socket_path(context.gest_dir().as_deref(), &data_dir);

    let csrf_key = resolve_csrf_key(serve_config.csrf_signing_key())?;

    crate::web::serve(
      context.store().clone(),
      project_id.clone(),
      addr,
      context.gest_dir().clone(),
      Some(socket_path),
      debounce_ms,
      csrf_key,
      cache_dir,
    )
    .await
    .map_err(std::io::Error::other)?;

    Ok(())
  }
}

/// Merge the generated `csrf_signing_key` into the existing `[serve]` table
/// and rewrite the config file preserving other keys.
fn persist_csrf_key(path: &Path, hex: &str) -> Result<(), Error> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  let mut root: Table = if path.is_file() {
    let content = fs::read_to_string(path)?;
    toml::from_str(&content).map_err(config::Error::from)?
  } else {
    Table::new()
  };

  let serve_entry = root
    .entry("serve".to_owned())
    .or_insert_with(|| Value::Table(Table::new()));
  if !serve_entry.is_table() {
    *serve_entry = Value::Table(Table::new());
  }
  let serve_table = serve_entry.as_table_mut().expect("just set to a table");
  serve_table.insert("csrf_signing_key".to_owned(), Value::String(hex.to_owned()));

  let content = toml::to_string_pretty(&Value::Table(root))?;
  fs::write(path, content)?;
  Ok(())
}

/// Resolve the CSRF signing key, generating and persisting one if absent.
///
/// When the `[serve].csrf_signing_key` config value is missing or malformed,
/// a fresh 32-byte HMAC key is generated and written to the global config
/// file so subsequent boots reuse the same key. If the global config path
/// cannot be resolved (e.g. because `$HOME` and `$XDG_CONFIG_HOME` are both
/// unset) the key is still generated in-memory -- outstanding cookies become
/// invalid on the next restart, matching the semantics of an ephemeral key.
fn resolve_csrf_key(configured: Option<&str>) -> Result<CsrfKey, Error> {
  if let Some(hex) = configured {
    match CsrfKey::from_hex(hex) {
      Ok(key) => return Ok(key),
      Err(err) => log::warn!("serve: invalid csrf_signing_key in config: {err}; regenerating"),
    }
  }

  let key = CsrfKey::generate();
  match resolve_global_config_path() {
    Ok(path) => {
      if let Err(err) = persist_csrf_key(&path, &key.to_hex()) {
        log::warn!("serve: failed to persist csrf_signing_key to {}: {err}", path.display(),);
      } else {
        log::info!("serve: generated and persisted csrf signing key at {}", path.display());
      }
    }
    Err(err) => log::warn!("serve: could not resolve global config path for csrf key: {err}"),
  }
  Ok(key)
}

/// Resolve the global config file path, honoring `$GEST_CONFIG`.
fn resolve_global_config_path() -> Result<PathBuf, Error> {
  GEST_CONFIG
    .value()
    .ok()
    .or_else(|| dir_spec::config_home().map(|path| path.join("gest/config.toml")))
    .ok_or_else(|| config::Error::XDGDirNotFound("config").into())
}
