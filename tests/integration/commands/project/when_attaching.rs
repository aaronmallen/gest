use crate::support::helpers::GestCmd;

fn primary_project_id(g: &GestCmd) -> String {
  let output = g
    .cmd()
    .args(["project", "--json"])
    .output()
    .expect("project --json failed");
  assert!(output.status.success(), "project --json should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  parsed["id"].as_str().expect("project id").to_string()
}

#[test]
fn it_attaches_additional_project() {
  let g = GestCmd::new();
  let primary_id = primary_project_id(&g);
  let extra_dir = g.temp_dir_path().join("extra");
  std::fs::create_dir_all(&extra_dir).expect("failed to create extra dir");

  // Run attach from the extra directory pointing at the primary project.
  let output = g
    .raw_cmd()
    .current_dir(&extra_dir)
    .env("NO_COLOR", "1")
    .args(["project", "attach", &primary_id])
    .output()
    .expect("project attach failed");

  assert!(
    output.status.success(),
    "project attach should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_errors_on_duplicate_attach() {
  let g = GestCmd::new();
  let primary_id = primary_project_id(&g);
  let extra_dir = g.temp_dir_path().join("dup-attach");
  std::fs::create_dir_all(&extra_dir).expect("failed to create extra dir");

  g.raw_cmd()
    .current_dir(&extra_dir)
    .env("NO_COLOR", "1")
    .args(["project", "attach", &primary_id])
    .assert()
    .success();

  // Attaching the same directory twice should error.
  let output = g
    .raw_cmd()
    .current_dir(&extra_dir)
    .env("NO_COLOR", "1")
    .args(["project", "attach", &primary_id])
    .output()
    .expect("project attach failed");

  assert!(!output.status.success(), "duplicate attach should exit non-zero");
}

#[test]
fn it_lists_attached_projects() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("list-extra");
  g.init_extra_project(&extra_dir);

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list --json failed");

  assert!(output.status.success(), "project list --json should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  let arr = parsed.as_array().expect("list should be an array");
  assert!(
    arr.len() >= 2,
    "expected at least two projects after init_extra_project, got: {stdout}"
  );
}
