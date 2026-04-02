use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_artifact_from_ndjson_stdin() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "create", "--batch"])
    .write_stdin("{\"title\":\"Art A\",\"body\":\"# Art A\\nbody\"}\n")
    .assert()
    .success();

  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Art A"));
}

#[test]
fn it_outputs_one_json_object_per_created_artifact() {
  let env = GestCmd::new();

  let output = env
    .cmd()
    .args(["artifact", "create", "--batch"])
    .write_stdin("{\"title\":\"Art A\",\"body\":\"# Art A\\nbody\"}\n")
    .output()
    .expect("failed to run artifact create --batch");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 1, "expected 1 output line, got: {lines:?}");

  let parsed: serde_json::Value =
    serde_json::from_str(lines[0]).unwrap_or_else(|e| panic!("line is not valid JSON ({e}): {}", lines[0]));
  assert!(parsed["id"].is_string(), "expected 'id' field in: {parsed}");
  assert!(parsed["title"].is_string(), "expected 'title' field in: {parsed}");
}

#[test]
fn it_outputs_a_bare_id_per_line_with_quiet_flag() {
  let env = GestCmd::new();

  let output = env
    .cmd()
    .args(["artifact", "create", "--batch", "-q"])
    .write_stdin("{\"title\":\"Art A\",\"body\":\"# Art A\\nbody\"}\n")
    .output()
    .expect("failed to run artifact create --batch -q");

  assert!(output.status.success());

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 1, "expected 1 ID line, got: {lines:?}");
  assert!(
    !lines[0].contains('{'),
    "expected a bare ID but got JSON-like output: {}",
    lines[0]
  );
  assert!(!lines[0].is_empty(), "expected a non-empty ID line");
}

#[test]
fn it_errors_with_line_number_on_invalid_json() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "create", "--batch"])
    .write_stdin("{\"title\":\"Good Art\",\"body\":\"body\"}\nnot-valid-json\n")
    .assert()
    .failure()
    .stderr(predicate::str::contains("line 2"));
}
