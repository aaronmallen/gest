use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_prints_version_string() {
  let env = GestCmd::new();

  env
    .run(["version"])
    .assert()
    .success()
    .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn it_prints_author_attribution() {
  let env = GestCmd::new();

  env
    .run(["version"])
    .assert()
    .success()
    .stdout(predicate::str::contains("@aaronmallen"));
}
