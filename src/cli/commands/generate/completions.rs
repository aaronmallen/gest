use std::io;

use clap::{Args, CommandFactory, ValueEnum};
use clap_complete::Shell;

use crate::Result;

/// Generate shell completion scripts
///
/// Prints a completion script to stdout. Redirect the output to the appropriate
/// file for your shell:
///
///   gest generate completions --shell zsh > ~/.zfunc/_gest
///   gest generate completions --shell bash > /etc/bash_completion.d/gest
///   gest generate completions --shell fish > ~/.config/fish/completions/gest.fish
#[derive(Args, Debug)]
pub struct Command {
  /// Target shell
  #[arg(long)]
  shell: ShellArg,
}

#[derive(Clone, Debug, ValueEnum)]
enum ShellArg {
  Bash,
  Elvish,
  Fish,
  #[value(name = "powershell")]
  PowerShell,
  Zsh,
}

impl From<ShellArg> for Shell {
  fn from(arg: ShellArg) -> Self {
    match arg {
      ShellArg::Bash => Shell::Bash,
      ShellArg::Elvish => Shell::Elvish,
      ShellArg::Fish => Shell::Fish,
      ShellArg::PowerShell => Shell::PowerShell,
      ShellArg::Zsh => Shell::Zsh,
    }
  }
}

impl Command {
  pub fn call(&self) -> Result<()> {
    let mut cmd = crate::cli::Cli::command();
    let shell: Shell = self.shell.clone().into();
    clap_complete::generate(shell, &mut cmd, "gest", &mut io::stdout());
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use clap::CommandFactory;
  use clap_complete::Shell;

  fn generate_to_vec(shell: Shell) -> Vec<u8> {
    let mut cmd = crate::cli::Cli::command();
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut cmd, "gest", &mut buf);
    buf
  }

  #[test]
  fn test_generate_bash_completions() {
    let output = generate_to_vec(Shell::Bash);
    assert!(!output.is_empty());
  }

  #[test]
  fn test_generate_elvish_completions() {
    let output = generate_to_vec(Shell::Elvish);
    assert!(!output.is_empty());
  }

  #[test]
  fn test_generate_fish_completions() {
    let output = generate_to_vec(Shell::Fish);
    assert!(!output.is_empty());
  }

  #[test]
  fn test_generate_powershell_completions() {
    let output = generate_to_vec(Shell::PowerShell);
    assert!(!output.is_empty());
  }

  #[test]
  fn test_generate_zsh_completions() {
    let output = generate_to_vec(Shell::Zsh);
    assert!(!output.is_empty());
  }

  #[test]
  fn test_version_flag_in_completions() {
    let output = generate_to_vec(Shell::Bash);
    let text = String::from_utf8(output).expect("valid utf8");
    assert!(
      text.contains("--version"),
      "completions should include the custom --version flag"
    );
  }
}
