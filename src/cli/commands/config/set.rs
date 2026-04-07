use clap::Args;

use crate::{AppContext, cli::Error, ui::components::SuccessMessage};

/// Set a configuration value.
#[derive(Args, Debug)]
pub struct Command {
  /// The configuration key.
  key: String,
  /// The value to set.
  value: String,
}

impl Command {
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    // TODO: write to config file
    let message = SuccessMessage::new("config set is not yet implemented");
    println!("{message}");
    Ok(())
  }
}
