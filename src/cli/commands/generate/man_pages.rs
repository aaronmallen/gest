use std::path::{Path, PathBuf};

use clap::{Args, Command as ClapCommand, CommandFactory};

use crate::cli;

/// Write roff man page files for all commands to a directory.
#[derive(Args, Debug)]
#[command(name = "man-pages")]
pub struct Command {
  /// Directory to write man page files into.
  #[arg(long)]
  output_dir: PathBuf,
}

impl Command {
  /// Create the output directory and generate all man pages.
  pub fn call(&self) -> cli::Result<()> {
    std::fs::create_dir_all(&self.output_dir)?;

    let cmd = crate::cli::Cli::command();
    generate_man_pages(&cmd, &self.output_dir, "gest")?;
    Ok(())
  }
}

/// Recursively render a `.1` man page for `cmd` and each visible subcommand.
fn generate_man_pages(cmd: &ClapCommand, dir: &Path, prefix: &str) -> cli::Result<()> {
  let man = clap_mangen::Man::new(cmd.clone());
  let filename = format!("{prefix}.1");
  let path = dir.join(&filename);
  let mut buf = Vec::new();
  man.render(&mut buf)?;
  std::fs::write(&path, buf)?;

  for subcmd in cmd.get_subcommands() {
    if subcmd.is_hide_set() {
      continue;
    }
    let sub_prefix = format!("{prefix}-{}", subcmd.get_name());
    generate_man_pages(subcmd, dir, &sub_prefix)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use clap::CommandFactory;

  use super::generate_man_pages;

  mod generate_man_pages {
    use super::*;

    #[test]
    fn it_generates_valid_man_page_content() {
      let dir = tempfile::tempdir().expect("create temp dir");
      let cmd = crate::cli::Cli::command();
      generate_man_pages(&cmd, dir.path(), "gest").expect("generate man pages");

      let content = std::fs::read_to_string(dir.path().join("gest.1")).expect("read man page");
      assert!(content.contains(".TH"), "man page should contain .TH header");
    }

    #[test]
    fn it_writes_man_page_files() {
      let dir = tempfile::tempdir().expect("create temp dir");
      let cmd = crate::cli::Cli::command();
      generate_man_pages(&cmd, dir.path(), "gest").expect("generate man pages");

      let root = dir.path().join("gest.1");
      assert!(root.exists(), "root man page should exist");

      let task = dir.path().join("gest-task.1");
      assert!(task.exists(), "gest-task man page should exist");
    }
  }
}
