use std::net::IpAddr;

use clap::Args;

use crate::{
  cli::{self, AppContext},
  ui::composites::success_message::SuccessMessage,
};

/// Start a local web server for browsing gest entities.
#[derive(Debug, Args)]
pub struct Command {
  /// Address to bind to (overrides config).
  #[arg(long = "bind", short = 'b')]
  bind_address: Option<IpAddr>,
  /// Port to listen on (overrides config).
  #[arg(long)]
  port: Option<u16>,
  /// Do not automatically open the browser.
  #[arg(long)]
  no_open: bool,
}

impl Command {
  /// Resolve effective settings from config + CLI flags and start the server.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let serve_config = ctx.settings.serve();
    let bind_address = self.bind_address.unwrap_or_else(|| serve_config.bind_address());
    let port = self.port.unwrap_or_else(|| serve_config.port());
    let open = !self.no_open && serve_config.open();

    let state = crate::server::ServerState::new(ctx.settings.clone());

    let rt = tokio::runtime::Runtime::new().map_err(|e| cli::Error::Runtime(e.to_string()))?;
    rt.block_on(async {
      let app = crate::server::router(state);
      let addr = std::net::SocketAddr::from((bind_address, port));
      let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| cli::Error::Runtime(e.to_string()))?;

      let url = format!("http://{bind_address}:{port}");
      let msg = SuccessMessage::new(format!("listening on {url}"), &ctx.theme);
      println!("{msg}");

      if open {
        let _ = open_browser(&url);
      }

      axum::serve(listener, app)
        .await
        .map_err(|e| cli::Error::Runtime(e.to_string()))
    })
  }
}

/// Attempt to open the given URL in the default browser.
fn open_browser(url: &str) -> std::io::Result<()> {
  #[cfg(target_os = "macos")]
  {
    std::process::Command::new("open").arg(url).spawn()?;
  }
  #[cfg(target_os = "linux")]
  {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
  }
  #[cfg(target_os = "windows")]
  {
    std::process::Command::new("cmd").args(["/C", "start", url]).spawn()?;
  }
  Ok(())
}
