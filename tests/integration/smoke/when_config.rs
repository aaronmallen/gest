use std::fs;

use crate::support::helpers::GestCmd;

#[test]
fn it_shows_the_resolved_config() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["config", "show"])
    .output()
    .expect("config show failed to run");

  assert!(output.status.success(), "config show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("[log]"), "got: {stdout}");
}

#[test]
fn it_shows_active_config_file_paths() {
  let g = GestCmd::new();
  let project_config = g.temp_dir_path().join(".gest.toml");
  fs::write(&project_config, "[log]\nlevel = \"info\"\n").expect("write project config");

  let output = g
    .cmd()
    .args(["config", "show"])
    .output()
    .expect("config show failed to run");

  assert!(output.status.success(), "config show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("project config"), "got: {stdout}");
  assert!(
    stdout.contains(&project_config.display().to_string()),
    "expected project config path in output, got: {stdout}"
  );
}

#[test]
fn it_shows_config_as_json() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["config", "show", "--json"])
    .output()
    .expect("config show --json failed to run");

  assert!(output.status.success(), "config show --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert!(parsed["log"].is_object(), "expected log table in JSON output");
}

#[test]
fn it_gets_a_config_value() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["config", "get", "log.level"])
    .output()
    .expect("config get failed to run");

  assert!(output.status.success(), "config get exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("warn"), "got: {stdout}");
}

#[test]
fn it_gets_a_config_value_as_json() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["config", "get", "log.level", "--json"])
    .output()
    .expect("config get --json failed to run");

  assert!(output.status.success(), "config get --json exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
  assert_eq!(parsed.as_str(), Some("warn"));
}
