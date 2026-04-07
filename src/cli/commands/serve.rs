use std::net::SocketAddr;

use clap::Args;

use crate::{AppContext, cli::Error};

/// Start the web dashboard server.
#[derive(Args, Debug)]
pub struct Command {
  /// File watcher debounce in milliseconds.
  #[arg(long, default_value = "300")]
  debounce_ms: u64,
  /// The host to bind to.
  #[arg(long, alias = "bind", default_value = "127.0.0.1")]
  host: String,
  /// Suppress automatic browser opening.
  #[arg(long)]
  no_open: bool,
  /// The port to bind to.
  #[arg(long, short, default_value = "2300")]
  port: u16,
}

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let project_id = context.project_id().as_ref().ok_or(Error::UninitializedProject)?;
    let addr: SocketAddr = format!("{}:{}", self.host, self.port)
      .parse()
      .map_err(|e: std::net::AddrParseError| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let url = format!("http://{addr}");
    println!("  starting gest dashboard at {url}");

    if !self.no_open
      && let Err(e) = open::that(&url)
    {
      log::warn!("failed to open browser: {e}");
    }

    crate::web::serve(
      context.store().clone(),
      project_id.clone(),
      addr,
      context.gest_dir().clone(),
      self.debounce_ms,
    )
    .await
    .map_err(std::io::Error::other)?;

    Ok(())
  }
}
