use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_prints_help_with_no_args() {
  let env = GestCmd::new();

  env.cmd().assert().success().stdout(predicate::str::contains("gest"));
}

#[test]
fn it_prints_help_with_help_flag() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage"));
}

#[test]
fn it_prints_version_with_short_flag() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("-V")
    .assert()
    .success()
    .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn it_prints_version_with_long_flag() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn it_rejects_invalid_command() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("invalid-command")
    .assert()
    .code(2)
    .stderr(predicate::str::contains("error"));
}

#[test]
fn it_rejects_invalid_flag() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("--invalid-flag")
    .assert()
    .code(2)
    .stderr(predicate::str::contains("error"));
}
