use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_errors_on_unknown_key() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["config", "get", "nonexistent.key"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Unknown config key"));
}

#[test]
fn it_gets_a_config_value() {
  let env = GestCmd::new();

  // First set a value so we can read it back.
  env
    .cmd()
    .args(["config", "set", "log.level", "info"])
    .assert()
    .success();

  env
    .cmd()
    .args(["config", "get", "log.level"])
    .assert()
    .success()
    .stdout(predicate::str::contains("info"));
}
