use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_accepts_comma_separated_tags() {
  let env = GestCmd::new();

  env
    .run(&["task", "create", "Tagged task", "--tag", "rust,cli"])
    .success()
    .stdout(predicate::str::contains("created task"));

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("failed to run task list");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("\"rust\""), "expected 'rust' tag in output: {stdout}");
  assert!(stdout.contains("\"cli\""), "expected 'cli' tag in output: {stdout}");
}

#[test]
fn it_accepts_repeated_tag_flags() {
  let env = GestCmd::new();

  env
    .run(&["task", "create", "Tagged task", "--tag", "rust", "--tag", "cli"])
    .success()
    .stdout(predicate::str::contains("created task"));

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("failed to run task list");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("\"rust\""), "expected 'rust' tag in output: {stdout}");
  assert!(stdout.contains("\"cli\""), "expected 'cli' tag in output: {stdout}");
}

#[test]
fn it_accepts_tags_alias() {
  let env = GestCmd::new();

  env
    .run(&["task", "create", "Tagged task", "--tags", "rust,cli"])
    .success()
    .stdout(predicate::str::contains("created task"));
}
