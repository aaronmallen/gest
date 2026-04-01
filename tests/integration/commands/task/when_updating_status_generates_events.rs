use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_task_and_get_id(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["task", "create", title])
    .output()
    .expect("failed to run gest task create");

  assert!(output.status.success(), "task create failed");

  let stdout = String::from_utf8(output.stdout).expect("stdout is not valid utf8");
  let first_line = stdout.lines().next().expect("no output from task create");
  first_line
    .split_whitespace()
    .last()
    .expect("no ID in create output")
    .to_string()
}

/// Read the task TOML file, checking both the active and resolved directories.
///
/// The short ID prefix returned by `task create` uniquely identifies the file; this helper
/// scans the tasks directory for a file whose stem starts with that prefix.
fn read_task_toml(env: &GestCmd, short_id: &str) -> String {
  // Try active directory first, then resolved.
  for subpath in &["tasks", "tasks/resolved"] {
    let dir = env.temp_dir_path().join(".gest").join(subpath);
    if let Ok(entries) = std::fs::read_dir(&dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
          if stem.starts_with(short_id) {
            return std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
          }
        }
      }
    }
  }
  panic!("could not find TOML file for task with short ID '{short_id}'");
}

#[test]
fn it_generates_no_event_when_status_is_unchanged() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "No-op task");

  // Update to the same status the task already has — no event should be generated.
  env.run(&["task", "update", &id, "--status", "open"]).success();

  let content = read_task_toml(&env, &id);
  assert!(
    !content.contains("status-change"),
    "expected no status-change event in the TOML file, got:\n{content}"
  );
}

#[test]
fn it_persists_the_event_so_task_show_json_includes_it() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "JSON event task");

  env.run(&["task", "update", &id, "--status", "in-progress"]).success();

  env
    .run(&["task", "show", &id, "--json"])
    .success()
    .stdout(predicate::str::contains("status-change"));
}

#[test]
fn it_records_a_status_change_event_in_the_task_file() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Event task");

  // Update status from the default 'open' to 'in-progress', which should generate an event.
  env.run(&["task", "update", &id, "--status", "in-progress"]).success();

  let content = read_task_toml(&env, &id);
  assert!(
    content.contains("status-change"),
    "expected a status-change event in the TOML file, got:\n{content}"
  );
}

#[test]
fn it_records_the_correct_from_and_to_statuses_in_the_event() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Status from-to task");

  env.run(&["task", "update", &id, "--status", "in-progress"]).success();

  let content = read_task_toml(&env, &id);
  assert!(
    content.contains("open"),
    "expected 'open' as the from-status in the event, got:\n{content}"
  );
  assert!(
    content.contains("in-progress"),
    "expected 'in-progress' as the to-status in the event, got:\n{content}"
  );
}
