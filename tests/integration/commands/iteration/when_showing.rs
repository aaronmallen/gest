use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_iteration(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Output: "Created iteration <id>"
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

#[test]
fn it_shows_an_iteration() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env
    .cmd()
    .args(["iteration", "show", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Sprint 1"));
}

#[test]
fn it_errors_on_nonexistent() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["iteration", "show", "doesnotexist00000000000000000000"])
    .assert()
    .failure();
}
