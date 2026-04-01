use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_iteration(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

fn create_artifact(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["artifact", "create", "--title", title, "--body", "Some content"])
    .output()
    .expect("failed to run gest artifact create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  // Output first line: "  ✓  created artifact  <8-char-id>"
  stdout
    .lines()
    .next()
    .and_then(|line| line.split_whitespace().last())
    .expect("no output from artifact create")
    .to_string()
}

#[test]
fn it_links_iteration_to_artifact() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");
  let artifact_id = create_artifact(&env, "Design Doc");

  env
    .cmd()
    .args(["iteration", "link", &iter_id, "relates-to", &artifact_id, "--artifact"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Linked"));
}
