use predicates::prelude::*;

use crate::support::helpers::{GestCmd, extract_id_from_create_output};

fn create_iteration(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

fn create_task(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["task", "create", title])
    .output()
    .expect("failed to run gest task create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  extract_id_from_create_output(&stdout).unwrap_or_else(|| panic!("could not extract task ID from output:\n{stdout}"))
}

#[test]
fn it_cancels_an_iteration() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env
    .cmd()
    .args(["iteration", "cancel", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Cancelled iteration"));
}

#[test]
fn it_cascades_to_tasks() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");
  let task_id = create_task(&env, "Task A");

  env.run(&["iteration", "add", &iter_id, &task_id]);

  env.run(&["iteration", "cancel", &iter_id]);

  env
    .cmd()
    .args(["task", "show", &task_id, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"cancelled\""));
}

#[test]
fn it_outputs_json() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env
    .cmd()
    .args(["iteration", "cancel", &id, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"cancelled\""));
}

#[test]
fn it_outputs_quiet() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  let output = env
    .cmd()
    .args(["iteration", "cancel", &id, "--quiet"])
    .output()
    .expect("failed to run cancel");

  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
  assert!(!stdout.is_empty());
}
