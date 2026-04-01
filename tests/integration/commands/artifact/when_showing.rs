use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_errors_on_nonexistent() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "show", "kkkkkkkkk"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("not found"));
}

#[test]
fn it_shows_an_artifact() {
  let env = GestCmd::new();

  let id = env.create_artifact("Show Me", "artifact body content");

  env
    .cmd()
    .args(["artifact", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Show Me"));
}
