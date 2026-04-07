use assert_cmd::Command;

use crate::support::helpers::GestCmd;

/// Build a second `gest init` command against the same data store as `g` but
/// running from `dir`. Used to seed a second project entry in the shared store.
fn init_extra_project(g: &GestCmd, dir: &std::path::Path) {
  std::fs::create_dir_all(dir).expect("failed to create extra project dir");
  let data_dir = g.temp_dir_path().join(".gest-data");
  let state_dir = g.temp_dir_path().join(".gest-state");
  let config = g.temp_dir_path().join("gest.toml");
  let project_dir = dir.join(".gest");
  std::fs::create_dir_all(&project_dir).expect("failed to create extra .gest dir");

  Command::cargo_bin("gest")
    .expect("gest binary not found")
    .current_dir(dir)
    .env("GEST_CONFIG", config)
    .env("GEST_STORAGE__DATA_DIR", data_dir)
    .env("GEST_PROJECT_DIR", project_dir)
    .env("GEST_STATE_DIR", state_dir)
    .env("NO_COLOR", "1")
    .args(["init"])
    .assert()
    .success();
}

#[test]
fn it_lists_projects_in_grid_format() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["project", "list"])
    .output()
    .expect("project list failed to run");

  assert!(
    output.status.success(),
    "project list exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("projects"), "missing projects header in: {stdout}");
  assert!(stdout.contains("1 project"), "missing count summary in: {stdout}");
  let root = g.temp_dir_path().display().to_string();
  assert!(stdout.contains(&root), "missing project root in: {stdout}");
}

#[test]
fn it_lists_projects_as_json() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list --json failed to run");

  assert!(output.status.success(), "project list --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 1, "expected one project, got: {stdout}");
  assert!(arr[0]["id"].is_string(), "project should have id: {stdout}");
  assert!(arr[0]["root"].is_string(), "project should have root: {stdout}");
}

#[test]
fn it_shows_current_project() {
  let g = GestCmd::new();

  let output = g.cmd().args(["project"]).output().expect("project show failed to run");

  assert!(
    output.status.success(),
    "project show exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("id"), "missing id field in: {stdout}");
  assert!(stdout.contains("root"), "missing root field in: {stdout}");
  let root = g.temp_dir_path().display().to_string();
  assert!(stdout.contains(&root), "missing project root in: {stdout}");
}

#[test]
fn it_shows_current_project_as_json() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["project", "--json"])
    .output()
    .expect("project --json failed to run");

  assert!(output.status.success(), "project --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert!(parsed["id"].is_string(), "project should have id: {stdout}");
  assert!(parsed["root"].is_string(), "project should have root: {stdout}");
}

#[test]
fn it_shows_json_root_value_matches_actual_path() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["project", "--json"])
    .output()
    .expect("project --json failed to run");

  assert!(output.status.success(), "project --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let root_in_json = parsed["root"].as_str().expect("root should be a string");
  // Canonicalize both sides to account for symlinks (e.g. /var -> /private/var on macOS).
  let expected_root = g
    .temp_dir_path()
    .canonicalize()
    .unwrap_or_else(|_| g.temp_dir_path().to_path_buf())
    .display()
    .to_string();
  let root_in_json_canonical = std::path::Path::new(root_in_json)
    .canonicalize()
    .unwrap_or_else(|_| std::path::PathBuf::from(root_in_json))
    .display()
    .to_string();
  assert_eq!(
    root_in_json_canonical, expected_root,
    "JSON root should equal the project temp dir"
  );
}

#[test]
fn it_shows_json_id_is_consistent_with_list_json_id() {
  let g = GestCmd::new();

  let list_out = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list --json failed to run");
  assert!(list_out.status.success(), "project list --json exited non-zero");
  let list_stdout = String::from_utf8_lossy(&list_out.stdout);
  let list_arr: serde_json::Value = serde_json::from_str(&list_stdout).expect("list output should be valid JSON");
  let list_id = list_arr[0]["id"].as_str().expect("list entry should have id");

  let show_out = g
    .cmd()
    .args(["project", "--json"])
    .output()
    .expect("project --json failed to run");
  assert!(show_out.status.success(), "project --json exited non-zero");
  let show_stdout = String::from_utf8_lossy(&show_out.stdout);
  let show_obj: serde_json::Value = serde_json::from_str(&show_stdout).expect("show output should be valid JSON");
  let show_id = show_obj["id"].as_str().expect("show should have id");

  assert_eq!(list_id, show_id, "id from list and show should match");
}

#[test]
fn it_lists_multiple_projects_with_plural_summary() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("other-project");
  init_extra_project(&g, &extra_dir);

  let output = g
    .cmd()
    .args(["project", "list"])
    .output()
    .expect("project list failed to run");

  assert!(
    output.status.success(),
    "project list exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("2 projects"), "expected plural summary in: {stdout}");
}

#[test]
fn it_lists_multiple_projects_shows_all_roots() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("second-project");
  init_extra_project(&g, &extra_dir);

  let output = g
    .cmd()
    .args(["project", "list"])
    .output()
    .expect("project list failed to run");

  assert!(
    output.status.success(),
    "project list exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let first_root = g.temp_dir_path().display().to_string();
  let second_root = extra_dir.display().to_string();
  assert!(stdout.contains(&first_root), "missing first project root in: {stdout}");
  assert!(
    stdout.contains(&second_root),
    "missing second project root in: {stdout}"
  );
}

#[test]
fn it_lists_multiple_projects_as_json_with_correct_count() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("third-project");
  init_extra_project(&g, &extra_dir);

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list --json failed to run");

  assert!(output.status.success(), "project list --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  let arr = parsed.as_array().expect("output should be a JSON array");
  assert_eq!(arr.len(), 2, "expected two projects in JSON array, got: {stdout}");
  for entry in arr {
    assert!(entry["id"].is_string(), "each project should have id field");
    assert!(entry["root"].is_string(), "each project should have root field");
  }
}

#[test]
fn it_list_human_output_is_not_valid_json() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["project", "list"])
    .output()
    .expect("project list failed to run");

  assert!(output.status.success(), "project list exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
    "human-readable list output should not parse as JSON, got: {stdout}"
  );
}

#[test]
fn it_show_human_output_is_not_valid_json() {
  let g = GestCmd::new();

  let output = g.cmd().args(["project"]).output().expect("project show failed to run");

  assert!(output.status.success(), "project show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
    "human-readable show output should not parse as JSON, got: {stdout}"
  );
}
