use std::fs;

use toml::{Table, Value};

use crate::support::helpers::{GestCmd, strip_ansi};

#[test]
fn it_emits_a_success_message_with_the_resolved_path() {
  let g = GestCmd::new();
  let expected_path = g.temp_dir_path().join(".gest.toml");

  let output = g
    .cmd()
    .args(["config", "set", "log.level", "debug"])
    .output()
    .expect("config set failed to run");

  assert!(
    output.status.success(),
    "config set exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
  assert!(stdout.contains("project"), "missing scope label: {stdout}");
  assert!(
    stdout.contains(expected_path.to_str().unwrap()),
    "missing resolved path: {stdout}"
  );
}

#[test]
fn it_infers_bool_int_and_string_value_types() {
  let g = GestCmd::new();
  let config_path = g.temp_dir_path().join(".gest.toml");

  g.cmd()
    .args(["config", "set", "flags.enabled", "true"])
    .assert()
    .success();
  g.cmd().args(["config", "set", "limits.count", "42"]).assert().success();
  g.cmd()
    .args(["config", "set", "limits.ratio", "1.5"])
    .assert()
    .success();
  g.cmd()
    .args(["config", "set", "labels.name", "hello"])
    .assert()
    .success();

  let content = fs::read_to_string(&config_path).expect("config file should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");

  let flags = parsed.get("flags").and_then(Value::as_table).unwrap();
  assert_eq!(flags.get("enabled"), Some(&Value::Boolean(true)));

  let limits = parsed.get("limits").and_then(Value::as_table).unwrap();
  assert_eq!(limits.get("count"), Some(&Value::Integer(42)));
  assert_eq!(limits.get("ratio"), Some(&Value::Float(1.5)));

  let labels = parsed.get("labels").and_then(Value::as_table).unwrap();
  assert_eq!(labels.get("name"), Some(&Value::String("hello".to_string())));
}

#[test]
fn it_round_trips_storage_cache_dir_through_set_and_get() {
  let g = GestCmd::new();
  let cache_dir = g.temp_dir_path().join("custom-cache");
  let cache_dir_str = cache_dir.to_str().unwrap().to_string();

  g.cmd()
    .args(["config", "set", "storage.cache_dir", &cache_dir_str])
    .assert()
    .success();

  let config_path = g.temp_dir_path().join(".gest.toml");
  let content = fs::read_to_string(&config_path).expect("config file should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");
  let storage = parsed.get("storage").and_then(Value::as_table).unwrap();
  assert_eq!(storage.get("cache_dir"), Some(&Value::String(cache_dir_str.clone())));

  let output = g
    .cmd()
    .args(["config", "get", "storage.cache_dir"])
    .output()
    .expect("config get failed to run");

  assert!(
    output.status.success(),
    "config get exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains(&cache_dir_str),
    "expected output to contain {cache_dir_str:?}, got: {stdout}"
  );
}

#[test]
fn it_sets_a_nested_dot_path_key() {
  let g = GestCmd::new();
  let config_path = g.temp_dir_path().join(".gest.toml");

  g.cmd().args(["config", "set", "a.b.c", "hello"]).assert().success();

  let content = fs::read_to_string(&config_path).expect("config file should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");

  let a = parsed.get("a").and_then(Value::as_table).unwrap();
  let b = a.get("b").and_then(Value::as_table).unwrap();
  assert_eq!(b.get("c"), Some(&Value::String("hello".to_string())));
}

#[test]
fn it_sets_a_project_scalar_and_creates_gest_toml() {
  let g = GestCmd::new();
  let config_path = g.temp_dir_path().join(".gest.toml");

  g.cmd().args(["config", "set", "log.level", "debug"]).assert().success();

  assert!(config_path.is_file(), ".gest.toml should exist");
  let content = fs::read_to_string(&config_path).expect("config file should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");
  let log = parsed.get("log").and_then(Value::as_table).unwrap();
  assert_eq!(log.get("level"), Some(&Value::String("debug".to_string())));
}

#[test]
fn it_updates_an_existing_key_without_clobbering_siblings() {
  let g = GestCmd::new();
  let config_path = g.temp_dir_path().join(".gest.toml");

  fs::write(
    &config_path,
    "[log]\nlevel = \"warn\"\nformat = \"json\"\n\n[storage]\ndata_dir = \"/tmp/data\"\n",
  )
  .expect("failed to seed .gest.toml");

  g.cmd().args(["config", "set", "log.level", "debug"]).assert().success();

  let content = fs::read_to_string(&config_path).expect("config file should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");

  let log = parsed.get("log").and_then(Value::as_table).unwrap();
  assert_eq!(log.get("level"), Some(&Value::String("debug".to_string())));
  assert_eq!(log.get("format"), Some(&Value::String("json".to_string())));

  let storage = parsed.get("storage").and_then(Value::as_table).unwrap();
  assert_eq!(storage.get("data_dir"), Some(&Value::String("/tmp/data".to_string())));
}

#[test]
fn it_walks_ancestors_and_writes_to_the_most_local_project_config_file() {
  let g = GestCmd::new();
  let ancestor_config = g.temp_dir_path().join(".gest.toml");
  fs::write(&ancestor_config, "[log]\nlevel = \"warn\"\n").expect("seed ancestor config");

  let nested = g.temp_dir_path().join("sub/dir");
  fs::create_dir_all(&nested).expect("create nested dir");

  g.cmd()
    .current_dir(&nested)
    .args(["config", "set", "log.level", "debug"])
    .assert()
    .success();

  let content = fs::read_to_string(&ancestor_config).expect("ancestor config should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");
  let log = parsed.get("log").and_then(Value::as_table).unwrap();
  assert_eq!(log.get("level"), Some(&Value::String("debug".to_string())));

  assert!(
    !nested.join(".gest.toml").exists(),
    "should not have created a nested .gest.toml"
  );
}

#[test]
fn it_writes_to_the_global_config_path_when_global_flag_is_set() {
  let g = GestCmd::new();
  let global_path = g.temp_dir_path().join("gest.toml");

  g.cmd()
    .args(["config", "set", "--global", "foo", "bar"])
    .assert()
    .success();

  assert!(global_path.is_file(), "global config file should exist");
  let content = fs::read_to_string(&global_path).expect("global config should exist");
  let parsed: Table = toml::from_str(&content).expect("config should be valid TOML");
  assert_eq!(parsed.get("foo"), Some(&Value::String("bar".to_string())));
}
