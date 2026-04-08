use std::net::{IpAddr, SocketAddr};

use clap::Args;

use crate::{AppContext, cli::Error};

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
    let socket_path = crate::web::reload_socket_path(context.gest_dir().as_deref(), &data_dir);

    crate::web::serve(
      context.store().clone(),
      project_id.clone(),
      addr,
      context.gest_dir().clone(),
      Some(socket_path),
      debounce_ms,
    )
    .await
    .map_err(std::io::Error::other)?;

    Ok(())
  }
}
