use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_all_tasks_from_ndjson_stdin() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["task", "create", "--batch"])
    .write_stdin("{\"title\":\"Task A\"}\n{\"title\":\"Task B\"}\n")
    .assert()
    .success();

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("failed to run task list --json");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output is not valid JSON");

  let tasks = parsed.as_array().expect("expected a JSON array");
  assert_eq!(tasks.len(), 2, "expected exactly 2 tasks, got: {tasks:?}");

  let titles: Vec<&str> = tasks.iter().filter_map(|t| t["title"].as_str()).collect();
  assert!(titles.contains(&"Task A"), "Task A not found in {titles:?}");
  assert!(titles.contains(&"Task B"), "Task B not found in {titles:?}");
}

#[test]
fn it_outputs_one_json_object_per_created_task() {
  let env = GestCmd::new();

  let output = env
    .cmd()
    .args(["task", "create", "--batch"])
    .write_stdin("{\"title\":\"Task A\"}\n{\"title\":\"Task B\"}\n")
    .output()
    .expect("failed to run task create --batch");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 2, "expected 2 output lines, got: {lines:?}");

  for line in &lines {
    let parsed: serde_json::Value =
      serde_json::from_str(line).unwrap_or_else(|e| panic!("line is not valid JSON ({e}): {line}"));
    assert!(parsed["id"].is_string(), "expected 'id' field in: {parsed}");
    assert!(parsed["title"].is_string(), "expected 'title' field in: {parsed}");
  }
}

#[test]
fn it_outputs_bare_ids_per_line_with_quiet_flag() {
  let env = GestCmd::new();

  let output = env
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin("{\"title\":\"Task A\"}\n{\"title\":\"Task B\"}\n")
    .output()
    .expect("failed to run task create --batch -q");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 2, "expected 2 ID lines, got: {lines:?}");

  for line in &lines {
    assert!(
      !line.contains('{'),
      "expected a bare ID but got JSON-like output: {line}"
    );
    assert!(!line.is_empty(), "expected a non-empty ID line");
  }
}

#[test]
fn it_errors_with_line_number_on_invalid_json() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["task", "create", "--batch"])
    .write_stdin("{\"title\":\"Good Task\"}\nnot-valid-json\n")
    .assert()
    .failure()
    .stderr(predicate::str::contains("line 2"));
}
