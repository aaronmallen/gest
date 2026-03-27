use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_prints_help_and_exits_successfully() {
  let env = GestCmd::new();

  env
    .run(["self-update", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Update gest"));
}

#[test]
fn it_shows_target_flag_in_help() {
  let env = GestCmd::new();

  env
    .run(["self-update", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("--target"));
}
