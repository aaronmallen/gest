use std::fs;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_artifact() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["artifact", "create", "My spec", "--body", "The body."])
    .output()
    .expect("artifact create failed to run");

  assert!(output.status.success(), "artifact create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("created artifact"), "got: {stdout}");
  assert!(stdout.contains("My spec"), "got: {stdout}");
}

#[test]
fn it_infers_title_from_first_heading_when_piping_stdin() {
  let g = GestCmd::new();
  let input = "# Piped artifact title\n\nbody goes here.\n";

  let output = g
    .cmd()
    .args(["artifact", "create"])
    .write_stdin(input)
    .output()
    .expect("artifact create failed to run");

  assert!(
    output.status.success(),
    "artifact create exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("created artifact"), "got: {stdout}");
  assert!(stdout.contains("Piped artifact title"), "got: {stdout}");
}

#[test]
fn it_reads_body_from_source_file_with_explicit_title() {
  let g = GestCmd::new();
  let source_path = g.temp_dir_path().join("doc.md");
  fs::write(&source_path, "# Heading in source\n\nsome content\n").expect("failed to write source file");

  let output = g
    .cmd()
    .args([
      "artifact",
      "create",
      "Explicit title",
      "--source",
      source_path.to_str().expect("path utf8"),
    ])
    .output()
    .expect("artifact create failed to run");

  assert!(
    output.status.success(),
    "artifact create exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Explicit title"), "got: {stdout}");
  assert!(
    !stdout.contains("Heading in source"),
    "explicit title should win over source heading, got: {stdout}"
  );
}

#[test]
fn it_errors_when_stdin_has_no_heading_and_no_title_arg() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["artifact", "create"])
    .write_stdin("no heading here\njust body text\n")
    .output()
    .expect("artifact create failed to run");

  assert!(
    !output.status.success(),
    "artifact create without title or heading should fail"
  );
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("heading") || stderr.contains("title"),
    "expected error mentioning heading/title, got: {stderr}"
  );
}
