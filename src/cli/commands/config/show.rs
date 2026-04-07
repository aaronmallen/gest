use clap::Args;

use crate::{AppContext, cli::Error};

/// Show the current configuration.
#[derive(Args, Debug)]
pub struct Command;

impl Command {
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    let toml = toml::to_string_pretty(context.settings()).map_err(std::io::Error::other)?;
    println!("{toml}");
    Ok(())
  }
}
