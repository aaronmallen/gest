use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_exits_successfully() {
  let env = GestCmd::new();

  env.cmd().arg("init").assert().success();
}

#[test]
fn it_prints_already_initialized_message() {
  let env = GestCmd::new();

  env
    .cmd()
    .arg("init")
    .assert()
    .success()
    .stdout(predicate::str::contains("already initialized").or(predicate::str::contains("Already")));
}
