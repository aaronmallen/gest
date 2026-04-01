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
fn it_omits_resolved_at_from_a_newly_created_task_file() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Unresolved task");

  let content = read_task_toml(&env, &id);
  assert!(
    !content.contains("resolved_at"),
    "expected 'resolved_at' to be absent from a new task's TOML, got:\n{content}"
  );
}

#[test]
fn it_writes_resolved_at_as_an_rfc3339_timestamp_after_resolving() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Task to resolve");

  env.run(&["task", "update", &id, "--status", "done"]).success();

  let content = read_task_toml(&env, &id);

  assert!(
    content.contains("resolved_at"),
    "expected 'resolved_at' to be present in a resolved task's TOML, got:\n{content}"
  );

  // Extract the value and confirm it is a non-empty RFC 3339 timestamp, not an empty string.
  // A valid RFC 3339 datetime always contains a 'T' separator between date and time.
  let resolved_at_line = content
    .lines()
    .find(|line| line.contains("resolved_at"))
    .unwrap_or_else(|| panic!("could not find 'resolved_at' line in:\n{content}"));

  assert!(
    !resolved_at_line.contains("\"\""),
    "expected a non-empty timestamp for 'resolved_at', got:\n{resolved_at_line}"
  );
  assert!(
    resolved_at_line.contains('T'),
    "expected an RFC 3339 timestamp (containing 'T') for 'resolved_at', got:\n{resolved_at_line}"
  );
}
