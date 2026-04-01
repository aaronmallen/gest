use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_shows_version() {
  let env = GestCmd::new_uninit();

  env
    .raw_cmd()
    .args(["version"])
    .assert()
    .success()
    .stdout(predicate::str::contains("v0.3.5"));
}

#[test]
fn it_shows_version_with_flag() {
  let env = GestCmd::new_uninit();

  env
    .raw_cmd()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains("v0.3.5"));
}
