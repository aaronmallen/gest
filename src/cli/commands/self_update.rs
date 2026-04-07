//! The `self-update` subcommand — downloads and installs the latest release from GitHub.

use clap::Args;
use self_update::backends::github::Update;

use crate::{AppContext, cli::Error, ui::components::SuccessMessage};

/// Download and install the latest release from GitHub.
#[derive(Args, Debug)]
pub struct Command {
  /// Pin to a specific release version (e.g. `0.5.0`).
  #[arg(long)]
  target: Option<String>,
}

impl Command {
  /// Check for updates and, if a newer version is available, download and
  /// replace the current binary.
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = self.target.clone();

    let status = tokio::task::spawn_blocking(move || {
      let mut builder = Update::configure();
      builder
        .repo_owner("aaronmallen")
        .repo_name("gest")
        .bin_name("gest")
        .current_version(current_version)
        .show_download_progress(true)
        .no_confirm(true);

      if let Some(ref version) = target {
        builder.target_version_tag(&format!("v{version}"));
      }

      builder.build().and_then(|updater| updater.update())
    })
    .await
    .map_err(std::io::Error::other)?
    .map_err(std::io::Error::other)?;

    if status.updated() {
      let message = SuccessMessage::new("updated gest")
        .field("previous version", current_version)
        .field("new version", status.version());
      println!("{message}");
    } else {
      let message = SuccessMessage::new("gest is already up to date").field("version", current_version);
      println!("{message}");
    }

    Ok(())
  }
}
