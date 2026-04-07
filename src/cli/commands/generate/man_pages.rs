use std::path::PathBuf;

use clap::{Args, CommandFactory};
use clap_mangen::Man;

use crate::{
  AppContext,
  cli::{App, Error},
  ui::components::SuccessMessage,
};

/// Generate man pages.
#[derive(Args, Debug)]
pub struct Command {
  /// Output directory for man pages.
  #[arg(default_value = "man")]
  output_dir: PathBuf,
}

impl Command {
  pub async fn call(&self, _context: &AppContext) -> Result<(), Error> {
    std::fs::create_dir_all(&self.output_dir)?;

    let cmd = App::command();
    let man = Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let path = self.output_dir.join("gest.1");
    std::fs::write(&path, buffer)?;

    let message = SuccessMessage::new("generated man pages").field("output", path.display().to_string());
    println!("{message}");
    Ok(())
  }
}
