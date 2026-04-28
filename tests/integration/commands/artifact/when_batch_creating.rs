use serde_json::Value;

use crate::support::helpers::GestCmd;

#[test]
fn it_emits_themed_count_in_default_mode() {
  let g = GestCmd::new();
  let input = "{\"title\":\"first batch artifact\"}\n{\"title\":\"second batch artifact\"}\n";

  let output = g
    .cmd()
    .args(["artifact", "create", "--batch"])
    .write_stdin(input)
    .output()
    .expect("artifact create --batch failed to run");

  assert!(
    output.status.success(),
    "artifact create --batch should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("batch created artifacts"),
    "expected themed message, got: {stdout}"
  );
  assert!(stdout.contains("count"), "expected count field, got: {stdout}");
  assert!(stdout.contains('2'), "expected count value 2, got: {stdout}");
}

#[test]
fn it_emits_one_short_id_per_line_in_quiet_mode() {
  let g = GestCmd::new();
  let input = "{\"title\":\"quiet artifact one\"}\n{\"title\":\"quiet artifact two\"}\n";

  let output = g
    .cmd()
    .args(["artifact", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("artifact create --batch -q failed to run");

  assert!(
    output.status.success(),
    "artifact create --batch -q should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

  assert_eq!(lines.len(), 2, "expected exactly 2 lines, got: {stdout}");
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
  let input = "{\"title\":\"json artifact one\"}\n{\"title\":\"json artifact two\"}\n";

  let output = g
    .cmd()
    .args(["artifact", "create", "--batch", "--json"])
    .write_stdin(input)
    .output()
    .expect("artifact create --batch --json failed to run");

  assert!(
    output.status.success(),
    "artifact create --batch --json should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
  let arr = json.as_array().expect("expected JSON array");

  assert_eq!(arr.len(), 2);
  assert_eq!(arr[0]["title"], "json artifact one");
  assert_eq!(arr[1]["title"], "json artifact two");
  assert!(arr[0].get("tags").is_some(), "expected envelope shape with tags");
  assert!(
    arr[0].get("relationships").is_some(),
    "expected envelope shape with relationships"
  );
}

#[test]
fn it_prefers_json_when_quiet_and_json_are_combined() {
  let g = GestCmd::new();
  let input = "{\"title\":\"both flags artifact\"}\n";

  let output = g
    .cmd()
    .args(["artifact", "create", "--batch", "-q", "-j"])
    .write_stdin(input)
    .output()
    .expect("artifact create --batch -q -j failed to run");

  assert!(
    output.status.success(),
    "artifact create --batch -q -j should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: Value = serde_json::from_str(&stdout).expect("expected JSON output (json wins over quiet)");
  let arr = json.as_array().expect("expected array");

  assert_eq!(arr.len(), 1);
  assert_eq!(arr[0]["title"], "both flags artifact");
}

#[test]
fn it_rolls_back_entire_batch_on_invalid_input() {
  let g = GestCmd::new();
  let input = "{\"title\":\"valid artifact before failure\"}\n{not valid json}\n";

  let output = g
    .cmd()
    .args(["artifact", "create", "--batch", "-q"])
    .write_stdin(input)
    .output()
    .expect("artifact create --batch failed to run");

  assert!(
    !output.status.success(),
    "batch with malformed input row should exit non-zero"
  );

  let list = g
    .cmd()
    .args(["artifact", "list", "--json"])
    .output()
    .expect("artifact list failed to run");
  assert!(list.status.success(), "artifact list should succeed");
  let stdout = String::from_utf8_lossy(&list.stdout);

  assert!(
    !stdout.contains("valid artifact before failure"),
    "artifact from rolled-back batch should not exist, got: {stdout}"
  );
}
