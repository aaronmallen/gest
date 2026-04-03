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
fn it_reopens_a_cancelled_iteration() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env.run(&["iteration", "cancel", &id]);

  env
    .cmd()
    .args(["iteration", "reopen", &id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Reopened iteration"));
}

#[test]
fn it_restores_cancelled_tasks_to_open() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");
  let task_id = create_task(&env, "Task A");

  env.run(&["iteration", "add", &iter_id, &task_id]);
  env.run(&["iteration", "cancel", &iter_id]);
  env.run(&["iteration", "reopen", &iter_id]);

  env
    .cmd()
    .args(["task", "show", &task_id, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"open\""));
}

#[test]
fn it_leaves_done_tasks_unchanged() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");
  let task_open = create_task(&env, "Open Task");
  let task_done = create_task(&env, "Done Task");

  env.run(&["iteration", "add", &iter_id, &task_open]);
  env.run(&["iteration", "add", &iter_id, &task_done]);
  env.run(&["task", "complete", &task_done]);

  env.run(&["iteration", "cancel", &iter_id]);
  env.run(&["iteration", "reopen", &iter_id]);

  env
    .cmd()
    .args(["task", "show", &task_open, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"open\""));

  env
    .cmd()
    .args(["task", "show", &task_done, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"done\""));
}

#[test]
fn it_outputs_json() {
  let env = GestCmd::new();
  let id = create_iteration(&env, "Sprint 1");

  env.run(&["iteration", "cancel", &id]);

  env
    .cmd()
    .args(["iteration", "reopen", &id, "--json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"status\": \"active\""));
}
