//! Integration tests for archived-project filtering across list commands.
//!
//! Since the `project archive` CLI command does not exist yet, these tests
//! archive projects by running SQL directly against the local database file.

use std::process;

use chrono::Utc;

use crate::support::helpers::{GestCmd, strip_ansi};

/// Archive a project by its full ID using the sqlite3 CLI.
fn archive_via_sql(g: &GestCmd, project_id: &str) {
  let db = g.db_path();
  let now = Utc::now().to_rfc3339();
  let sql = format!("UPDATE projects SET archived_at = '{now}', updated_at = '{now}' WHERE id = '{project_id}';");
  let output = process::Command::new("sqlite3")
    .arg(db)
    .arg(sql)
    .output()
    .expect("sqlite3 should be available");
  assert!(
    output.status.success(),
    "sqlite3 archive failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

/// Extract the full project ID from `project list --json` output whose root
/// path ends with the given `suffix` (typically the last path component).
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
fn it_hides_archived_projects_from_default_list() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("to-archive");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "to-archive");
  archive_via_sql(&g, &id);

  let output = g
    .cmd()
    .args(["project", "list", "--json"])
    .output()
    .expect("project list failed");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  let arr = parsed.as_array().expect("JSON array");

  assert_eq!(arr.len(), 1, "archived project should be hidden, got: {stdout}");
  assert_ne!(
    arr[0]["id"].as_str().unwrap(),
    id,
    "the visible project should not be the archived one"
  );
}

#[test]
fn it_reveals_archived_projects_with_all_flag() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("to-archive");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "to-archive");
  archive_via_sql(&g, &id);

  let output = g
    .cmd()
    .args(["project", "list", "--all", "--json"])
    .output()
    .expect("project list --all failed");

  assert!(
    output.status.success(),
    "project list --all --json failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
  let arr = parsed.as_array().expect("JSON array");

  assert_eq!(
    arr.len(),
    2,
    "both projects should be visible with --all, got: {stdout}"
  );
}

#[test]
fn it_labels_archived_projects_in_human_output() {
  let g = GestCmd::new();
  let extra_dir = g.temp_dir_path().join("to-archive");
  g.init_extra_project(&extra_dir);

  let id = project_id_by_root_suffix(&g, "to-archive");
  archive_via_sql(&g, &id);

  let output = g
    .cmd()
    .args(["project", "list", "--all"])
    .output()
    .expect("project list --all failed");

  assert!(
    output.status.success(),
    "project list --all failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let plain = strip_ansi(&stdout);

  assert!(
    plain.contains("[archived]"),
    "archived project should have [archived] badge in: {plain}"
  );
  assert!(
    plain.contains("2 projects"),
    "summary should count both projects in: {plain}"
  );
}
