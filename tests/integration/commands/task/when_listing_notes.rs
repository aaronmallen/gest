use crate::support::helpers::GestCmd;

fn add_note(g: &GestCmd, task_id: &str, body: &str) -> String {
  let output = g
    .cmd()
    .args(["task", "note", "add", task_id, "-b", body, "--quiet"])
    .output()
    .expect("task note add failed");
  assert!(output.status.success());
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn it_lists_notes_in_order() {
  let g = GestCmd::new();
  let task_id = g.create_task("notable");
  add_note(&g, &task_id, "first body");
  add_note(&g, &task_id, "second body");

  let output = g
    .cmd()
    .args(["task", "note", "list", &task_id])
    .output()
    .expect("task note list failed");

  assert!(output.status.success(), "task note list should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("first body"), "missing first: {stdout}");
  assert!(stdout.contains("second body"), "missing second: {stdout}");
}

#[test]
fn it_lists_notes_as_json() {
  let g = GestCmd::new();
  let task_id = g.create_task("json notable");
  add_note(&g, &task_id, "json body");

  let output = g
    .cmd()
    .args(["task", "note", "list", &task_id, "--json"])
    .output()
    .expect("task note list --json failed");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  let arr = parsed.as_array().expect("array");
  assert!(!arr.is_empty(), "should contain at least one note, got: {stdout}");
}

#[test]
fn it_lists_empty_when_no_notes() {
  let g = GestCmd::new();
  let task_id = g.create_task("empty notes");

  let output = g
    .cmd()
    .args(["task", "note", "list", &task_id])
    .output()
    .expect("task note list failed");

  assert!(output.status.success(), "task note list should succeed");
}

#[test]
fn it_lists_notes_as_quiet_short_ids() {
  let g = GestCmd::new();
  let task_id = g.create_task("quiet notable");
  let first = add_note(&g, &task_id, "first body");
  let second = add_note(&g, &task_id, "second body");

  let output = g
    .cmd()
    .args(["task", "note", "list", &task_id, "--quiet"])
    .output()
    .expect("task note list --quiet failed");

  assert!(output.status.success(), "task note list --quiet should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();
  assert_eq!(lines.len(), 2, "expected one id per line, got: {stdout}");
  assert!(lines.contains(&first.as_str()), "missing first id {first}: {stdout}");
  assert!(lines.contains(&second.as_str()), "missing second id {second}: {stdout}");
}
