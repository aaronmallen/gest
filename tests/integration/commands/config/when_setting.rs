use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_sets_a_config_value() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["config", "set", "log.level", "warn"])
    .assert()
    .success()
    .stdout(predicate::str::contains("log.level"));
}
