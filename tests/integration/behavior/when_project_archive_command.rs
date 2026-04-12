//! Integration tests for `gest project archive` and `gest project unarchive` CLI commands.

use crate::support::helpers::{GestCmd, strip_ansi};

/// Extract the full project ID from `project list --json` output whose root
/// path ends with the given `suffix`.
fn project_id_by_root_suffix(g: &GestCmd, suffix: &str) -> String {
  let output = g
    .cmd()
    .args(["project", "list", "--all", "--json"])
    .output()
    .expect("project list --json failed");
  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  let arr = parsed.as_array().expect("JSON array");
  for entry in arr {
    if let Some(r) = entry["root"].as_str() {
      if r.ends_with(suffix) {
        return entry["id"].as_str().unwrap().to_string();
      }
    }
  }
  panic!("no project found with root ending in {suffix} in: {stdout}");
}

#[test]
fn it_archives_a_project_with_yes_flag() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("archivable");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "archivable");

  let output = g
    .cmd()
    .args(["project", "archive", &id, "--yes"])
    .output()
    .expect("project archive failed");

  assert!(
    output.status.success(),
    "project archive exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let plain = strip_ansi(&stdout);

  assert!(
    plain.contains("archived project"),
    "should show success message, got: {plain}"
  );
}

#[test]
fn it_hides_archived_project_from_default_list() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("to-hide");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "to-hide");

  g.cmd().args(["project", "archive", &id, "--yes"]).assert().success();

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  let arr = parsed.as_array().expect("JSON array");

  assert_eq!(
    arr.len(),
    1,
    "archived project should be hidden from default list, got: {stdout}"
  );
}

#[test]
fn it_detaches_workspaces_on_archive() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("ws-project");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "ws-project");

  // Attach a workspace to the extra project
  let ws_dir = g.temp_dir_path().join("my-workspace");
  std::fs::create_dir_all(&ws_dir).unwrap();

  let attach_output = std::process::Command::new(assert_cmd::cargo::cargo_bin("gest"))
    .current_dir(&ws_dir)
    .env("GEST_CONFIG", g.temp_dir_path().join("gest.toml"))
    .env("GEST_STORAGE__DATA_DIR", g.temp_dir_path().join(".gest-data"))
    .env("GEST_PROJECT_DIR", ws_dir.join(".gest"))
    .env("GEST_STATE_DIR", g.temp_dir_path().join(".gest-state"))
    .env("NO_COLOR", "1")
    .args(["project", "attach", &id])
    .output()
    .expect("project attach failed");
  assert!(
    attach_output.status.success(),
    "project attach failed: {}",
    String::from_utf8_lossy(&attach_output.stderr)
  );

  // Archive should report the workspace count
  let output = g
    .cmd()
    .args(["project", "archive", &id, "--yes"])
    .output()
    .expect("project archive failed");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let plain = strip_ansi(&stdout);

  assert!(
    plain.contains("workspaces detached"),
    "should report workspace detachment, got: {plain}"
  );
}

#[test]
fn it_prompts_without_yes_flag() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("prompt-test");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "prompt-test");

  // Without --yes and with stdin closed (EOF), the prompt defaults to No and
  // the command exits successfully without archiving.
  let output = g
    .cmd()
    .args(["project", "archive", &id])
    .output()
    .expect("project archive failed to run");

  assert!(
    output.status.success(),
    "command should exit cleanly even when declining"
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  let plain = strip_ansi(&stdout);
  assert!(
    !plain.contains("archived project"),
    "project should not be archived without confirmation, got: {plain}"
  );

  // Verify the project is still visible (not archived)
  let list_output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");
  let list_stdout = String::from_utf8_lossy(&list_output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&list_stdout).expect("valid JSON");
  assert_eq!(
    parsed.as_array().unwrap().len(),
    2,
    "both projects should still be visible after declined archive"
  );
}

#[test]
fn it_unarchives_a_project() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("to-unarchive");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "to-unarchive");

  // Archive first
  g.cmd().args(["project", "archive", &id, "--yes"]).assert().success();

  // Verify it is hidden
  let list_output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");
  let stdout = String::from_utf8_lossy(&list_output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  assert_eq!(parsed.as_array().unwrap().len(), 1, "should be hidden after archive");

  // Unarchive
  let output = g
    .cmd()
    .args(["project", "unarchive", &id])
    .output()
    .expect("project unarchive failed");

  assert!(
    output.status.success(),
    "project unarchive failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let plain = strip_ansi(&stdout);

  assert!(
    plain.contains("unarchived project"),
    "should show success message, got: {plain}"
  );
  assert!(
    plain.contains("Workspace paths are not automatically restored"),
    "should print workspace reattach hint, got: {plain}"
  );
}

#[test]
fn it_restores_project_to_default_list_after_unarchive() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("restore-test");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "restore-test");

  g.cmd().args(["project", "archive", &id, "--yes"]).assert().success();

  g.cmd().args(["project", "unarchive", &id]).assert().success();

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  let arr = parsed.as_array().expect("JSON array");

  assert_eq!(
    arr.len(),
    2,
    "both projects should be visible after unarchive, got: {stdout}"
  );
}
