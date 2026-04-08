use crate::support::helpers::GestCmd;

#[test]
fn it_caps_json_output() {
  let g = GestCmd::new();
  for i in 0..5 {
    g.create_task(&format!("JSON task {i}"));
  }

  let output = g
    .cmd()
    .args(["task", "list", "--limit", "3", "--json"])
    .output()
    .expect("task list --limit --json failed to run");

  assert!(output.status.success(), "task list --limit --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 3, "expected 3 tasks in JSON, got: {stdout}");
}

#[test]
fn it_caps_quiet_output() {
  let g = GestCmd::new();
  for i in 0..5 {
    g.create_task(&format!("Quiet task {i}"));
  }

  let output = g
    .cmd()
    .args(["task", "list", "--limit", "1", "--quiet"])
    .output()
    .expect("task list --limit --quiet failed to run");

  assert!(output.status.success(), "task list --limit --quiet exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
  assert_eq!(lines.len(), 1, "expected 1 ID line, got: {stdout}");
}

#[test]
fn it_caps_table_output() {
  let g = GestCmd::new();
  for i in 0..5 {
    g.create_task(&format!("Task {i}"));
  }

  let output = g
    .cmd()
    .args(["task", "list", "--limit", "2"])
    .output()
    .expect("task list --limit failed to run");

  assert!(output.status.success(), "task list --limit exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let task_lines = stdout.lines().filter(|l| l.contains("Task ")).count();
  assert_eq!(task_lines, 2, "expected 2 task lines, got: {stdout}");
}
