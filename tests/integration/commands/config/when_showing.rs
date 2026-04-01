use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_shows_config() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["config", "show"])
    .assert()
    .success()
    .stdout(predicate::str::contains("configuration"))
    .stdout(predicate::str::contains("data_dir"))
    .stdout(predicate::str::contains("log_level"));
}
