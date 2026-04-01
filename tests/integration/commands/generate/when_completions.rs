use assert_cmd::Command;
use predicates::prelude::*;

fn gest() -> Command {
  Command::cargo_bin("gest").expect("gest binary not found")
}

#[test]
fn it_generates_bash_completions() {
  gest()
    .args(["generate", "completions", "--shell", "bash"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn it_generates_zsh_completions() {
  gest()
    .args(["generate", "completions", "--shell", "zsh"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn it_errors_on_unknown_shell() {
  gest()
    .args(["generate", "completions", "--shell", "notashell"])
    .assert()
    .failure();
}
