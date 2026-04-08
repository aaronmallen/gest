use crate::support::helpers::GestCmd;

#[test]
fn it_caps_json_output() {
  let g = GestCmd::new();
  let extra = g.temp_dir_path().join("limit-json-extra-1");
  g.init_extra_project(&extra);
  let extra2 = g.temp_dir_path().join("limit-json-extra-2");
  g.init_extra_project(&extra2);

  let output = g
    .cmd()
    .args(["project", "list", "--limit", "2", "--json"])
    .output()
    .expect("project list --limit --json failed to run");

  assert!(output.status.success(), "project list --limit --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 2, "expected 2 projects in JSON, got: {stdout}");
}

#[test]
fn it_caps_table_output() {
  let g = GestCmd::new();
  let extra = g.temp_dir_path().join("limit-extra-1");
  g.init_extra_project(&extra);
  let extra2 = g.temp_dir_path().join("limit-extra-2");
  g.init_extra_project(&extra2);

  let output = g
    .cmd()
    .args(["project", "list", "--limit", "1"])
    .output()
    .expect("project list --limit failed to run");

  assert!(output.status.success(), "project list --limit exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  // The summary line uses the visible count
  assert!(stdout.contains("1 project"), "expected singular summary, got: {stdout}");
}
