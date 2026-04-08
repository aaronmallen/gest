use crate::support::helpers::GestCmd;

#[test]
fn it_caps_json_output() {
  let g = GestCmd::new();
  for i in 0..4 {
    g.create_iteration(&format!("JSON iteration {i}"));
  }

  let output = g
    .cmd()
    .args(["iteration", "list", "--limit", "1", "--json"])
    .output()
    .expect("iteration list --limit --json failed to run");

  assert!(output.status.success(), "iteration list --limit --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 1, "expected 1 iteration in JSON, got: {stdout}");
}

#[test]
fn it_caps_table_output() {
  let g = GestCmd::new();
  for i in 0..4 {
    g.create_iteration(&format!("Iteration {i}"));
  }

  let output = g
    .cmd()
    .args(["iteration", "list", "--limit", "2"])
    .output()
    .expect("iteration list --limit failed to run");

  assert!(output.status.success(), "iteration list --limit exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines = stdout.lines().filter(|l| l.contains("Iteration ")).count();
  assert_eq!(lines, 2, "expected 2 iteration lines, got: {stdout}");
}
