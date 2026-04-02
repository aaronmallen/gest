use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn gest() -> Command {
  Command::cargo_bin("gest").expect("gest binary not found")
}

#[test]
fn it_prints_a_success_message_containing_generated() {
  let output_dir = TempDir::new().expect("failed to create temp dir");

  gest()
    .args([
      "generate",
      "man-pages",
      "--output-dir",
      output_dir.path().to_str().expect("valid path"),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Generated"));
}

#[test]
fn it_prints_a_success_message_containing_man_pages() {
  let output_dir = TempDir::new().expect("failed to create temp dir");

  gest()
    .args([
      "generate",
      "man-pages",
      "--output-dir",
      output_dir.path().to_str().expect("valid path"),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("man pages"));
}
