use predicates::prelude::*;

use crate::support::helpers::GestCmd;

/// Extract the 8-character task ID from create output like "Created task abcdefgh".
fn extract_task_id(create_output: &str) -> String {
  create_output
    .trim()
    .strip_prefix("Created task ")
    .expect("expected 'Created task <id>' output")
    .to_string()
}

#[test]
fn it_sets_and_gets_metadata() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Meta Task", "-d", "metadata test"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env
    .run(["task", "meta", "set", &id, "priority", "high"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Set"));

  env
    .run(["task", "meta", "get", &id, "priority"])
    .assert()
    .success()
    .stdout(predicate::str::contains("high"));
}

#[test]
fn it_overwrites_existing_metadata() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "Overwrite Meta", "-d", "overwrite test"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env.run(["task", "meta", "set", &id, "level", "low"]).assert().success();

  env
    .run(["task", "meta", "set", &id, "level", "critical"])
    .assert()
    .success();

  env
    .run(["task", "meta", "get", &id, "level"])
    .assert()
    .success()
    .stdout(predicate::str::contains("critical"));
}

#[test]
fn it_errors_on_missing_metadata_key() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "No Meta", "-d", "no metadata"])
    .output()
    .expect("failed to create task");
  let id = extract_task_id(&String::from_utf8_lossy(&output.stdout));

  env.run(["task", "meta", "get", &id, "nonexistent"]).assert().failure();
}
