use std::io;

use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};

use crate::{
  AppContext,
  cli::{App, Error},
};

/// Generate shell completions.
#[derive(Args, Debug)]
pub struct Command {
  /// The shell to generate completions for.
  shell: Shell,
}

impl Command {
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    let mut cmd = App::command();
    generate(self.shell, &mut cmd, "gest", &mut io::stdout());
    Ok(())
  }
}
