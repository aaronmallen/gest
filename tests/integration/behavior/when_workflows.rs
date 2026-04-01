use predicates::prelude::*;

use crate::support::helpers::GestCmd;

/// Extract the first whitespace-delimited token that looks like a short ID
/// (8 lowercase hex-ish chars) from a create-command output string.
///
/// The create view renders: `✓  created <kind>  <id8chars>`
/// followed by field lines. We scan every word and return the first one
/// whose length is exactly 8 and consists only of alphanumeric characters.
fn extract_short_id(output: &str) -> Option<String> {
  for token in output.split_whitespace() {
    // Strip any ANSI escape sequences that might remain
    let clean: String = token.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_alphanumeric()) {
      return Some(clean);
    }
  }
  None
}

#[test]
fn it_creates_then_lists_tasks() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["task", "create", "My Workflow Task"])
    .assert()
    .success();

  env
    .cmd()
    .args(["task", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("My Workflow Task"));
}

#[test]
fn it_creates_then_shows_task() {
  let env = GestCmd::new();

  let create_output = env
    .cmd()
    .args(["task", "create", "Showable Task"])
    .assert()
    .success()
    .get_output()
    .stdout
    .clone();

  let text = String::from_utf8(create_output).expect("stdout is not valid UTF-8");
  let short_id = extract_short_id(&text).expect("could not find short ID in create output");

  env
    .cmd()
    .args(["task", "show", &short_id])
    .assert()
    .success()
    .stdout(predicate::str::contains("Showable Task"));
}

#[test]
fn it_creates_artifact_then_lists() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "My Design Spec",
      "--body",
      "# My Design Spec\n\nSpec content here.",
    ])
    .assert()
    .success();

  env
    .cmd()
    .args(["artifact", "list"])
    .assert()
    .success()
    .stdout(predicate::str::contains("My Design Spec"));
}
