//! The `version` subcommand — prints version and platform information.

use clap::Args;

use crate::{
  AppContext,
  cli::Error,
  ui::components::{Banner, UpdateNotice},
};

/// Print the current version, platform info, and check for available updates.
#[derive(Args, Debug)]
pub struct Command;

impl Command {
  /// Display the version banner including author, version details, and an
  /// update notice when a newer release exists on GitHub.
  ///
  /// The banner renders immediately so slow network responses can't block
  /// output. The GitHub release check runs concurrently and, if a newer
  /// version is found, an update notice is appended after the banner. If
  /// the check fails or the local version is already current, no update
  /// notice is shown.
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    // Kick off the GitHub release check on the blocking thread pool so
    // network latency doesn't block banner rendering. `self_update`'s
    // `fetch()` is synchronous and constructs its own internal runtime,
    // so it cannot run inside `tokio::spawn` without panicking on drop.
    let check_handle = tokio::task::spawn_blocking(|| {
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

    println!("{}", Banner::new().with_author().with_version());

    if let Ok(Some(new_version)) = check_handle.await {
      println!("\n{}", UpdateNotice::new(new_version));
    }

    Ok(())
  }
}
