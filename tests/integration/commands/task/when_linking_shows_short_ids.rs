use predicates::prelude::*;

use crate::support::helpers::GestCmd;

/// Create a task and return its full 32-character ID via `task list --json`.
fn create_task_and_get_full_id(env: &GestCmd, title: &str) -> String {
  env.cmd().args(["task", "create", title]).assert().success();

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .output()
    .expect("failed to run gest task list --json");

  assert!(output.status.success(), "task list --json failed");

  let json: serde_json::Value =
    serde_json::from_slice(&output.stdout).expect("task list --json output is not valid JSON");

  json
    .as_array()
    .expect("task list JSON should be an array")
    .iter()
    .find_map(|t| {
      if t["title"].as_str() == Some(title) {
        t["id"].as_str().map(str::to_string)
      } else {
        None
      }
    })
    .unwrap_or_else(|| panic!("could not find task with title '{title}' in task list JSON"))
}

#[test]
fn it_shows_short_ids_in_link_success_output() {
  let env = GestCmd::new();
  let full_id1 = create_task_and_get_full_id(&env, "Source task");
  let full_id2 = create_task_and_get_full_id(&env, "Target task");

  // Short IDs are the first 8 characters of the full 32-char ID.
  let short_id1 = &full_id1[..8];
  let short_id2 = &full_id2[..8];

  env
    .run(&["task", "link", &full_id1, "blocks", &full_id2])
    .success()
    .stdout(predicate::str::contains(short_id1))
    .stdout(predicate::str::contains(short_id2));
}

#[test]
fn it_does_not_show_full_ids_in_link_success_output() {
  let env = GestCmd::new();
  let full_id1 = create_task_and_get_full_id(&env, "Source task");
  let full_id2 = create_task_and_get_full_id(&env, "Target task");

  env
    .run(&["task", "link", &full_id1, "relates-to", &full_id2])
    .success()
    .stdout(predicate::str::contains(&full_id1).not())
    .stdout(predicate::str::contains(&full_id2).not());
}
