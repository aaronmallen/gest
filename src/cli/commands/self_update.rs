use clap::Args;

use crate::{
  config::Config,
  ui::{
    components::{AlreadyOnVersion, UpdateAvailable, UpdateCancelled, UpdateComplete, UpdatePrompt},
    theme::Theme,
  },
};

/// Binary name.
const BIN_NAME: &str = "gest";

/// Current version from Cargo.toml, used to detect whether an update is needed.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository name.
const REPO_NAME: &str = "gest";

/// GitHub repository owner.
const REPO_OWNER: &str = "aaronmallen";

/// Update gest to the latest (or a specific) release
#[derive(Debug, Args)]
pub struct Command {
  /// Pin to a specific version (bare semver, e.g. 1.2.3)
  #[arg(long)]
  target: Option<String>,
}

impl Command {
  pub fn call(&self, _config: &Config, theme: &Theme) -> crate::Result<()> {
    let releases = self_update::backends::github::ReleaseList::configure()
      .repo_owner(REPO_OWNER)
      .repo_name(REPO_NAME)
      .build()?
      .fetch()?;

    let latest = releases
      .first()
      .ok_or_else(|| crate::Error::generic("no releases found on GitHub"))?;

    let target_version = self.target.as_deref().unwrap_or(&latest.version);

    if target_version == CURRENT_VERSION {
      AlreadyOnVersion::new(target_version).write_to(&mut std::io::stdout(), theme)?;
      return Ok(());
    }

    // Prompt for confirmation
    UpdateAvailable::new(CURRENT_VERSION, target_version).write_to(&mut std::io::stdout())?;
    UpdatePrompt.write_to(&mut std::io::stdout())?;
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer != "y" && answer != "yes" {
      UpdateCancelled.write_to(&mut std::io::stdout())?;
      return Ok(());
    }

    let status = self_update::backends::github::Update::configure()
      .repo_owner(REPO_OWNER)
      .repo_name(REPO_NAME)
      .bin_name(BIN_NAME)
      .target_version_tag(target_version)
      .show_download_progress(true)
      .current_version(CURRENT_VERSION)
      .no_confirm(true)
      .build()?
      .update()?;

    UpdateComplete::new(status.version()).write_to(&mut std::io::stdout(), theme)?;

    Ok(())
  }
}
