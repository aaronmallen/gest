use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_sets_a_project_config_value() {
  let env = GestCmd::new();

  env
    .run(["config", "set", "harness.command", "codex"])
    .assert()
    .success()
    .stdout(predicate::str::contains("project"));
}

#[test]
fn it_sets_a_global_config_value() {
  let env = GestCmd::new();
  let xdg_temp = tempfile::tempdir().expect("failed to create temp dir for XDG_CONFIG_HOME");

  env
    .cmd()
    .args(["config", "set", "--global", "harness.command", "codex"])
    .env("XDG_CONFIG_HOME", xdg_temp.path())
    .assert()
    .success()
    .stdout(predicate::str::contains("global"));
}

#[test]
fn it_persists_a_set_value() {
  let env = GestCmd::new();

  env
    .run(["config", "set", "harness.command", "codex"])
    .assert()
    .success();

  env
    .run(["config", "get", "harness.command"])
    .assert()
    .success()
    .stdout(predicate::str::contains("codex"));
}
