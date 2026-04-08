use crate::support::helpers::GestCmd;

#[test]
fn it_caps_each_entity_type_independently() {
  let g = GestCmd::new();
  for i in 0..4 {
    g.create_task(&format!("findme task {i}"));
    g.create_artifact(&format!("findme artifact {i}"), "body");
    g.create_iteration(&format!("findme iteration {i}"));
  }

  let output = g
    .cmd()
    .args(["search", "findme", "--limit", "2", "--json"])
    .output()
    .expect("search --limit --json failed to run");

  assert!(output.status.success(), "search --limit --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

  let tasks = parsed["tasks"].as_array().expect("tasks should be array");
  let artifacts = parsed["artifacts"].as_array().expect("artifacts should be array");
  let iterations = parsed["iterations"].as_array().expect("iterations should be array");

  assert_eq!(tasks.len(), 2, "expected 2 tasks, got: {stdout}");
  assert_eq!(artifacts.len(), 2, "expected 2 artifacts, got: {stdout}");
  assert_eq!(iterations.len(), 2, "expected 2 iterations, got: {stdout}");
}
