use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_retrieves_a_config_value() {
  let env = GestCmd::new();

  env
    .run(["config", "get", "harness.command"])
    .assert()
    .success()
    .stdout(predicate::str::contains("claude"));
}

#[test]
fn it_fails_for_unknown_key() {
  let env = GestCmd::new();

  env.run(["config", "get", "nonexistent.key"]).assert().failure();
}
