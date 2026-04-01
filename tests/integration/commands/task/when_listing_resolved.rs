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

#[test]
fn it_lists_resolved_tasks() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Done task");

  env.run(&["task", "update", &id, "--status", "done"]).success();

  // Without --all, resolved tasks should not appear
  env
    .run(&["task", "list"])
    .success()
    .stdout(predicate::str::contains("Done task").not());

  // With --all, resolved tasks should appear
  env
    .run(&["task", "list", "--all"])
    .success()
    .stdout(predicate::str::contains("Done task"));
}
