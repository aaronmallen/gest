use std::path::PathBuf;

use clap::{Args, CommandFactory};

use crate::Result;

/// Generate man pages for all commands
///
/// Writes one .1 man page file per command and subcommand to the specified directory:
///
///   gest generate man-pages --output-dir ./man/
#[derive(Args, Debug)]
#[command(name = "man-pages")]
pub struct Command {
  /// Directory to write man page files to
  #[arg(long)]
  output_dir: PathBuf,
}

impl Command {
  pub fn call(&self) -> Result<()> {
    std::fs::create_dir_all(&self.output_dir)?;

    let cmd = crate::cli::Cli::command();
    generate_man_pages(&cmd, &self.output_dir, "gest")?;
    Ok(())
  }
}

fn generate_man_pages(cmd: &clap::Command, dir: &std::path::Path, prefix: &str) -> Result<()> {
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

  #[test]
  fn test_generate_man_pages_writes_files() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let cmd = crate::cli::Cli::command();
    generate_man_pages(&cmd, dir.path(), "gest").expect("generate man pages");

    let root = dir.path().join("gest.1");
    assert!(root.exists(), "root man page should exist");

    let artifact = dir.path().join("gest-artifact.1");
    assert!(artifact.exists(), "gest-artifact man page should exist");

    let task = dir.path().join("gest-task.1");
    assert!(task.exists(), "gest-task man page should exist");

    let generate = dir.path().join("gest-generate.1");
    assert!(generate.exists(), "gest-generate man page should exist");

    let completions = dir.path().join("gest-generate-completions.1");
    assert!(completions.exists(), "gest-generate-completions man page should exist");

    let man_pages = dir.path().join("gest-generate-man-pages.1");
    assert!(man_pages.exists(), "gest-generate-man-pages man page should exist");
  }

  #[test]
  fn test_man_page_content_is_valid() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let cmd = crate::cli::Cli::command();
    generate_man_pages(&cmd, dir.path(), "gest").expect("generate man pages");

    let content = std::fs::read_to_string(dir.path().join("gest.1")).expect("read man page");
    assert!(content.contains(".TH"), "man page should contain .TH header");
  }
}
