use crate::support::helpers::GestCmd;

fn task_priority(g: &GestCmd, task_id: &str) -> serde_json::Value {
  let show = g
    .cmd()
    .args(["task", "show", task_id, "--json"])
    .output()
    .expect("task show failed");
  assert!(show.status.success(), "task show exited non-zero");
  let stdout = String::from_utf8_lossy(&show.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  parsed["priority"].clone()
}

#[test]
fn it_creates_a_task_with_a_label_priority() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["task", "create", "labeled", "--priority", "high", "--json"])
    .output()
    .expect("task create failed to run");

  assert!(output.status.success(), "task create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(parsed["priority"], serde_json::json!(1));
}

#[test]
fn it_creates_a_task_with_an_integer_priority() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["task", "create", "numbered", "--priority", "1", "--json"])
    .output()
    .expect("task create failed to run");

  assert!(output.status.success(), "task create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(parsed["priority"], serde_json::json!(1));
}

#[test]
fn it_rejects_an_invalid_label() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["task", "create", "bogus", "--priority", "urgent"])
    .output()
    .expect("task create failed to run");

  assert!(!output.status.success(), "expected task create to fail");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("urgent"),
    "stderr should mention the bad value: {stderr}"
  );
  assert!(
    stderr.contains("critical") && stderr.contains("lowest"),
    "stderr should list the accepted forms: {stderr}"
  );
}

#[test]
fn it_updates_a_task_with_an_uppercase_label() {
  let g = GestCmd::new();
  let task_id = g.create_task("updatable");

  g.cmd()
    .args(["task", "update", &task_id, "--priority", "CRITICAL"])
    .assert()
    .success();

  assert_eq!(task_priority(&g, &task_id), serde_json::json!(0));
}
