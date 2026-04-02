use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_task_with_note(env: &GestCmd) -> (String, String) {
  env
    .cmd()
    .args(["task", "create", "Test Task", "--description", "A test task"])
    .assert()
    .success();

  let output = env
    .cmd()
    .args(["task", "list", "--json", "--all"])
    .output()
    .expect("failed to run task list");

  let tasks: serde_json::Value = serde_json::from_slice(&output.stdout).expect("failed to parse task list JSON");
  let task_id = tasks[0]["id"].as_str().expect("task id not found in JSON").to_string();

  env
    .cmd()
    .args([
      "task",
      "note",
      "add",
      &task_id,
      "--agent",
      "test-agent",
      "--body",
      "Alias test note",
    ])
    .assert()
    .success();

  let output = env
    .cmd()
    .args(["task", "note", "list", &task_id, "--json"])
    .output()
    .expect("failed to run note list");

  let notes: serde_json::Value = serde_json::from_slice(&output.stdout).expect("failed to parse note list JSON");
  let note_id = notes[0]["id"].as_str().expect("note id not found in JSON").to_string();

  (task_id, note_id)
}

#[test]
fn it_lists_notes_using_ls_alias() {
  let env = GestCmd::new();
  let (task_id, _) = create_task_with_note(&env);

  env
    .cmd()
    .args(["task", "note", "ls", &task_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Alias test note"));
}

#[test]
fn it_shows_a_note_using_view_alias() {
  let env = GestCmd::new();
  let (task_id, note_id) = create_task_with_note(&env);

  env
    .cmd()
    .args(["task", "note", "view", &task_id, &note_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Alias test note"));
}
