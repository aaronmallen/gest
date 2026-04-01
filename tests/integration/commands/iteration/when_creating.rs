use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_iteration() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["iteration", "create", "Sprint 1"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created iteration"));
}

#[test]
fn it_errors_without_title() {
  let env = GestCmd::new();

  env.cmd().args(["iteration", "create"]).assert().failure();
}
