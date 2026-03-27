use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_a_task_with_description_flag() {
  let env = GestCmd::new();

  env
    .run(["task", "create", "My Task", "-d", "A description"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Created task"));
}

#[test]
fn it_creates_a_task_via_stdin_pipe() {
  let env = GestCmd::new();

  env
    .run(["task", "create", "Stdin Task"])
    .write_stdin("description from stdin\n")
    .assert()
    .success()
    .stdout(predicate::str::contains("Created task"));
}

#[test]
fn it_outputs_an_id_on_creation() {
  let env = GestCmd::new();

  let output = env
    .run(["task", "create", "ID Task", "-d", "test"])
    .output()
    .expect("failed to run command");

  assert!(output.status.success());
  let stdout = String::from_utf8_lossy(&output.stdout);
  // Output format: "Created task <8-char-id>\n"
  // ID characters are in the range k-z
  let line = stdout.trim();
  assert!(
    line.starts_with("Created task "),
    "Expected 'Created task <id>' but got: {line}"
  );
  let id_part = line.strip_prefix("Created task ").unwrap();
  assert_eq!(id_part.len(), 8, "ID should be 8 characters, got: {id_part}");
  assert!(
    id_part.chars().all(|c| ('k'..='z').contains(&c)),
    "ID should only contain k-z, got: {id_part}"
  );
}
