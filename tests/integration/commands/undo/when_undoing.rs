use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_errors_with_no_history() {
  let env = GestCmd::new();

  // A freshly initialized project has no mutations to undo.
  env
    .run(&["undo"])
    .failure()
    .stderr(predicate::str::contains("Nothing to undo"));
}

#[test]
fn it_undoes_a_mutation() {
  let env = GestCmd::new();

  // Create a task so there is something to undo.
  env.run(&["task", "create", "Undo me"]).success();

  // Verify the task exists before undoing.
  env
    .run(&["task", "list"])
    .success()
    .stdout(predicate::str::contains("Undo me"));

  // Undo the create.
  env.run(&["undo"]).success().stdout(predicate::str::contains("Undid"));

  // The task should no longer appear in the list.
  env
    .run(&["task", "list"])
    .stdout(predicate::str::contains("no tasks found"));
}
