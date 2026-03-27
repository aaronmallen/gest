use crate::support::helpers::GestCmd;

#[test]
fn it_exits_successfully() {
  let env = GestCmd::new();

  env.run(["config", "show"]).assert().success();
}

#[test]
fn it_prints_configuration() {
  use predicates::prelude::*;

  let env = GestCmd::new();

  env
    .run(["config", "show"])
    .assert()
    .success()
    .stdout(predicate::str::contains("harness"));
}
