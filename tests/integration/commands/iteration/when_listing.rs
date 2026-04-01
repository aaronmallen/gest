use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_lists_iterations() {
  let env = GestCmd::new();

  env.cmd().args(["iteration", "create", "Sprint 1"]).assert().success();

  env
    .cmd()
    .args(["iteration", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Sprint 1"));
}

#[test]
fn it_lists_empty() {
  let env = GestCmd::new();

  env.cmd().args(["iteration", "list"]).assert().success();
}
