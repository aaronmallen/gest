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
fn it_untags_a_task() {
  let env = GestCmd::new();
  let id = create_task_and_get_id(&env, "Untag target");

  env.run(&["task", "tag", &id, "removeme"]).success();

  env
    .run(&["task", "show", &id])
    .success()
    .stdout(predicate::str::contains("removeme"));

  env.run(&["task", "untag", &id, "removeme"]).success();

  env
    .run(&["task", "show", &id])
    .success()
    .stdout(predicate::str::contains("removeme").not());
}
