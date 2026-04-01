use predicates::prelude::*;

use crate::support::helpers::GestCmd;

fn create_iteration_and_get_id(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  assert!(output.status.success(), "iteration create failed");

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

/// Read the iteration TOML file, checking both the active and resolved directories.
///
/// The short ID prefix returned by `iteration create` uniquely identifies the file; this helper
/// scans the iterations directory for a file whose stem starts with that prefix.
fn read_iteration_toml(env: &GestCmd, short_id: &str) -> String {
  for subpath in &["iterations", "iterations/resolved"] {
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
  panic!("could not find TOML file for iteration with short ID '{short_id}'");
}

#[test]
fn it_generates_no_event_when_status_is_unchanged() {
  let env = GestCmd::new();
  let id = create_iteration_and_get_id(&env, "No-op iteration");

  // Update to the same status the iteration already has — no event should be generated.
  env.run(&["iteration", "update", &id, "--status", "active"]).success();

  let content = read_iteration_toml(&env, &id);
  assert!(
    !content.contains("status-change"),
    "expected no status-change event in the TOML file, got:\n{content}"
  );
}

#[test]
fn it_persists_the_event_so_iteration_show_json_includes_it() {
  let env = GestCmd::new();
  let id = create_iteration_and_get_id(&env, "JSON event iteration");

  env.run(&["iteration", "update", &id, "--status", "failed"]).success();

  env
    .run(&["iteration", "show", &id, "--json"])
    .success()
    .stdout(predicate::str::contains("status-change"));
}

#[test]
fn it_records_a_status_change_event_in_the_iteration_file() {
  let env = GestCmd::new();
  let id = create_iteration_and_get_id(&env, "Event iteration");

  // Update status from the default 'active' to 'failed', which should generate an event.
  env.run(&["iteration", "update", &id, "--status", "failed"]).success();

  let content = read_iteration_toml(&env, &id);
  assert!(
    content.contains("status-change"),
    "expected a status-change event in the TOML file, got:\n{content}"
  );
}

#[test]
fn it_records_the_correct_from_and_to_statuses_in_the_event() {
  let env = GestCmd::new();
  let id = create_iteration_and_get_id(&env, "Status from-to iteration");

  env.run(&["iteration", "update", &id, "--status", "failed"]).success();

  let content = read_iteration_toml(&env, &id);
  assert!(
    content.contains("active"),
    "expected 'active' as the from-status in the event, got:\n{content}"
  );
  assert!(
    content.contains("failed"),
    "expected 'failed' as the to-status in the event, got:\n{content}"
  );
}
