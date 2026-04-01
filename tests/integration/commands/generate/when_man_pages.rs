use assert_cmd::Command;
use tempfile::TempDir;

fn gest() -> Command {
  Command::cargo_bin("gest").expect("gest binary not found")
}

#[test]
fn it_generates_man_pages() {
  let output_dir = TempDir::new().expect("failed to create temp dir");

  gest()
    .args([
      "generate",
      "man-pages",
      "--output-dir",
      output_dir.path().to_str().expect("valid path"),
    ])
    .assert()
    .success();

  assert!(
    output_dir.path().join("gest.1").exists(),
    "root man page should be created"
  );
}
