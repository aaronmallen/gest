use serde_json::Value;

use crate::support::helpers::GestCmd;

#[test]
fn it_emits_themed_count_in_default_mode() {
  let g = GestCmd::new();
  let input = "{\"title\":\"first batch task\"}\n{\"title\":\"second batch task\"}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch"])
    .write_stdin(input)
    .output()
    .expect("task create --batch failed to run");

  assert!(
    output.status.success(),
    "task create --batch should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("batch created"),
    "expected themed message, got: {stdout}"
  );
  assert!(stdout.contains("count"), "expected count field, got: {stdout}");
  assert!(stdout.contains('2'), "expected count value 2, got: {stdout}");
}

#[test]
fn it_emits_one_short_id_per_line_in_quiet_mode() {
  let g = GestCmd::new();
  let input = "{\"title\":\"quiet task one\"}\n{\"title\":\"quiet task two\"}\n{\"title\":\"quiet task three\"}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("task create --batch -q failed to run");

  assert!(
    output.status.success(),
    "task create --batch -q should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

  assert_eq!(lines.len(), 3, "expected exactly 3 lines, got: {stdout}");
  for line in &lines {
    assert!(
      line.chars().all(|c| c.is_ascii_alphanumeric()),
      "expected bare short_id on each line, got: {line:?}"
    );
  }
}

#[test]
fn it_emits_envelope_array_in_json_mode() {
  let g = GestCmd::new();
  let input = "{\"title\":\"json task one\"}\n{\"title\":\"json task two\"}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "--json"])
    .write_stdin(input)
    .output()
    .expect("task create --batch --json failed to run");

  assert!(
    output.status.success(),
    "task create --batch --json should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
  let arr = json.as_array().expect("expected JSON array");

  assert_eq!(arr.len(), 2);
  assert_eq!(arr[0]["title"], "json task one");
  assert_eq!(arr[1]["title"], "json task two");
  assert!(arr[0].get("tags").is_some(), "expected envelope shape with tags");
  assert!(
    arr[0].get("relationships").is_some(),
    "expected envelope shape with relationships"
  );
}

#[test]
fn it_prefers_json_when_quiet_and_json_are_combined() {
  let g = GestCmd::new();
  let input = "{\"title\":\"both flags task\"}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q", "-j"])
    .write_stdin(input)
    .output()
    .expect("task create --batch -q -j failed to run");

  assert!(
    output.status.success(),
    "task create --batch -q -j should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: Value = serde_json::from_str(&stdout).expect("expected JSON output (json wins over quiet)");
  let arr = json.as_array().expect("expected array");

  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["title"], "both flags task");
}

#[test]
fn it_rolls_back_entire_batch_on_invalid_input() {
  let g = GestCmd::new();
  let input = "{\"title\":\"valid task before failure\"}\n{not valid json}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("task create --batch failed to run");

  assert!(
    !output.status.success(),
    "batch with malformed input row should exit non-zero"
  );

  // Verify no tasks were created: list tasks and confirm the valid one is absent.
  let list = g
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("task list failed to run");
  assert!(list.status.success(), "task list should succeed");
  let stdout = String::from_utf8_lossy(&list.stdout);

  assert!(
    !stdout.contains("valid task before failure"),
    "task from rolled-back batch should not exist, got: {stdout}"
  );
}

#[test]
fn it_applies_tags_from_batch_input() {
  let g = GestCmd::new();
  let input = "{\"title\":\"tagged batch task\",\"tags\":[\"alpha\",\"beta\"]}\n";

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("task create --batch failed to run");

  assert!(
    output.status.success(),
    "task create --batch should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let task_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
  assert!(!task_id.is_empty(), "expected a task ID");

  let show = g
    .cmd()
    .args(["task", "show", &task_id, "--json"])
    .output()
    .expect("task show failed to run");
  let json: Value = serde_json::from_slice(&show.stdout).expect("valid JSON");
  let tags = json["tags"].as_array().expect("expected tags array");

  assert!(tags.contains(&Value::String("alpha".into())), "expected alpha tag");
  assert!(tags.contains(&Value::String("beta".into())), "expected beta tag");
}

#[test]
fn it_attaches_to_iteration_from_batch_input() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("batch iteration target");
  let input = format!("{{\"title\":\"iteration-bound task\",\"iteration\":\"{iter_id}\"}}\n");

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("task create --batch failed to run");

  assert!(
    output.status.success(),
    "task create --batch should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  // Confirm the iteration now contains the batch-created task by inspecting the
  // human-rendered show output, which surfaces aggregate task counts.
  let show = g
    .cmd()
    .args(["iteration", "show", &iter_id])
    .output()
    .expect("iteration show failed to run");
  let stdout = String::from_utf8_lossy(&show.stdout);

  assert!(
    stdout.contains("tasks") && stdout.contains('1'),
    "iteration should report a task count of 1, got: {stdout}"
  );
}

#[test]
fn it_creates_links_from_batch_input() {
  let g = GestCmd::new();
  let blocker = g.create_task("blocker task");
  let input = format!("{{\"title\":\"blocked task\",\"links\":[\"blocked-by:{blocker}\"]}}\n");

  let output = g
    .cmd()
    .args(["task", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("task create --batch failed to run");

  assert!(
    output.status.success(),
    "task create --batch should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let blocked_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

  let show = g
    .cmd()
    .args(["task", "show", &blocked_id, "--json"])
    .output()
    .expect("task show failed to run");
  let json: Value = serde_json::from_slice(&show.stdout).expect("valid JSON");
  let rels = json["relationships"].as_array().expect("expected relationships");

  assert!(
    !rels.is_empty(),
    "blocked task should have a relationship, got: {rels:?}"
  );
  assert!(
    rels.iter().any(|r| r["rel_type"] == "blocked-by"),
    "expected blocked-by relationship, got: {rels:?}"
  );
}
