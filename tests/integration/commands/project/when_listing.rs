use crate::support::helpers::GestCmd;

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
fn it_lists_multiple_projects_with_plural_summary() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("other-project");
  g.init_extra_project(&extra_dir);

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
  g.init_extra_project(&extra_dir);

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
  g.init_extra_project(&extra_dir);

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
