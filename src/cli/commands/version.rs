//! The `version` subcommand — prints version and platform information.

use clap::Args;

use crate::{AppContext, cli::Error, ui::components::Banner};

/// Print the current version, platform info, and check for available updates.
#[derive(Args, Debug)]
pub struct Command;

impl Command {
  /// Display the version banner including author, version details, and an
  /// update notice when a newer release exists on GitHub.
  ///
  /// The GitHub release check runs on a background task so the banner can
  /// render immediately if the network is slow. If the check fails or the
  /// local version is already current, no update notice is shown.
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    // Fetch the latest release from GitHub on a background task so network
    // latency doesn't block banner rendering.
    let check_handle = tokio::spawn(async {
      let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("aaronmallen")
        .repo_name("gest")
        .build()
        .ok()?
        .fetch()
        .ok()?;
      let latest = releases.first()?;
      let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
      let remote = semver::Version::parse(&latest.version).ok()?;
      if remote > current {
        Some(latest.version.clone())
      } else {
        None
      }
    });

    let new_version = check_handle.await.unwrap_or(None);
    println!("{}", Banner::new().with_author().with_version(new_version));
    Ok(())
  }
}
