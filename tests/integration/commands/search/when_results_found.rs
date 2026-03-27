use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_finds_a_task_by_title() {
  let env = GestCmd::new();

  env
    .run(["task", "create", "Unicorn migration plan", "-d", "move all unicorns"])
    .assert()
    .success();

  env
    .run(["search", "unicorn"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Unicorn migration plan"));
}

#[test]
fn it_finds_an_artifact_by_title() {
  let env = GestCmd::new();

  env
    .run([
      "artifact",
      "create",
      "-t",
      "Flamingo research notes",
      "-b",
      "body content here",
    ])
    .assert()
    .success();

  env
    .run(["search", "flamingo"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Flamingo research notes"));
}

#[test]
fn it_returns_task_matches_as_json() {
  let env = GestCmd::new();

  env
    .run([
      "task",
      "create",
      "Zeppelin refueling protocol",
      "-d",
      "fill with helium",
    ])
    .assert()
    .success();

  let output = env
    .run(["search", "--json", "zeppelin"])
    .output()
    .expect("failed to run search --json");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON output");

  let tasks = json["tasks"].as_array().expect("expected tasks array");
  assert_eq!(tasks.len(), 1, "expected exactly one task match");
  assert_eq!(tasks[0]["title"], "Zeppelin refueling protocol");
  assert!(tasks[0]["id"].is_string(), "expected id to be a string");
  assert_eq!(tasks[0]["status"], "open");
}

#[test]
fn it_returns_artifact_matches_as_json() {
  let env = GestCmd::new();

  env
    .run([
      "artifact",
      "create",
      "-t",
      "Platypus analysis report",
      "-b",
      "body of the report",
    ])
    .assert()
    .success();

  let output = env
    .run(["search", "--json", "platypus"])
    .output()
    .expect("failed to run search --json");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: serde_json::Value = serde_json::from_str(&stdout).expect("expected valid JSON output");

  let artifacts = json["artifacts"].as_array().expect("expected artifacts array");
  assert_eq!(artifacts.len(), 1, "expected exactly one artifact match");
  assert_eq!(artifacts[0]["title"], "Platypus analysis report");
  assert!(artifacts[0]["id"].is_string(), "expected id to be a string");
}
