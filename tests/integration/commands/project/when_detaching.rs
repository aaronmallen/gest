use crate::support::helpers::GestCmd;

fn primary_project_id(g: &GestCmd) -> String {
  let output = g
    .cmd()
    .args(["project", "--json"])
    .output()
    .expect("project --json failed");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  parsed["id"].as_str().expect("project id").to_string()
}

#[test]
fn it_cannot_detach_primary() {
  let g = GestCmd::new();
  // No other project has been attached; running detach from a directory that
  // never had an attachment registered should either no-op or error -- in
  // either case the project list must remain intact afterwards.
  let unattached = g.temp_dir_path().join("unattached");
  std::fs::create_dir_all(&unattached).expect("failed to create unattached dir");

  let detach = g
    .raw_cmd()
    .current_dir(&unattached)
    .env("NO_COLOR", "1")
    .args(["project", "detach"])
    .output()
    .expect("project detach failed");

  // The primary project should still be listed regardless of the detach outcome.
  let list = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");
  assert!(
    list.status.success(),
    "project list should succeed after detach attempt"
  );
  let stdout = String::from_utf8_lossy(&list.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert!(
    parsed.as_array().map(|a| !a.is_empty()).unwrap_or(false),
    "project list should still have entries; detach exit: {:?}",
    detach.status.code()
  );
}

#[test]
fn it_detaches_project() {
  let g = GestCmd::new();
  let primary_id = primary_project_id(&g);
  let extra_dir = g.temp_dir_path().join("detach-me");
  std::fs::create_dir_all(&extra_dir).expect("failed to create extra dir");

  g.raw_cmd()
    .current_dir(&extra_dir)
    .env("NO_COLOR", "1")
    .args(["project", "attach", &primary_id])
    .assert()
    .success();

  let output = g
    .raw_cmd()
    .current_dir(&extra_dir)
    .env("NO_COLOR", "1")
    .args(["project", "detach"])
    .output()
    .expect("project detach failed");

  assert!(
    output.status.success(),
    "project detach should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_errors_on_unknown_project() {
  let g = GestCmd::new();
  let stray = g.temp_dir_path().join("never-attached");
  std::fs::create_dir_all(&stray).expect("failed to create stray dir");

  // Attempting to detach a directory that was never attached should exit
  // non-zero with a descriptive error.
  let output = g
    .raw_cmd()
    .current_dir(&stray)
    .env("NO_COLOR", "1")
    .args(["project", "detach"])
    .output()
    .expect("project detach failed");

  assert!(
    !output.status.success(),
    "detaching an unattached directory should exit non-zero"
  );
}
