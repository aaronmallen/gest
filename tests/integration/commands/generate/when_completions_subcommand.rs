use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_outputs_bash_completions() {
  let env = GestCmd::new();

  env
    .run(["generate", "completions", "--shell", "bash"])
    .assert()
    .success()
    .stdout(predicate::str::contains("gest"));
}

#[test]
fn it_outputs_zsh_completions() {
  let env = GestCmd::new();

  env
    .run(["generate", "completions", "--shell", "zsh"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn it_outputs_fish_completions() {
  let env = GestCmd::new();

  env
    .run(["generate", "completions", "--shell", "fish"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}
