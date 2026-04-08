use crate::support::helpers::GestCmd;

#[test]
fn it_caps_json_output() {
  let g = GestCmd::new();
  for i in 0..4 {
    g.create_artifact(&format!("JSON artifact {i}"), "body");
  }

  let output = g
    .cmd()
    .args(["artifact", "list", "--limit", "1", "--json"])
    .output()
    .expect("artifact list --limit --json failed to run");

  assert!(output.status.success(), "artifact list --limit --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 1, "expected 1 artifact in JSON, got: {stdout}");
}

#[test]
fn it_caps_table_output() {
  let g = GestCmd::new();
  for i in 0..4 {
    g.create_artifact(&format!("Artifact {i}"), "body");
  }

  let output = g
    .cmd()
    .args(["artifact", "list", "--limit", "2"])
    .output()
    .expect("artifact list --limit failed to run");

  assert!(output.status.success(), "artifact list --limit exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let artifact_lines = stdout.lines().filter(|l| l.contains("Artifact ")).count();
  assert_eq!(artifact_lines, 2, "expected 2 artifact lines, got: {stdout}");
}
