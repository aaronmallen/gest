use crate::support::helpers::GestCmd;

#[test]
fn it_writes_man_page_files() {
  let env = GestCmd::new();
  let output_dir = tempfile::tempdir().expect("failed to create temp dir for man pages");

  env
    .run([
      "generate",
      "man-pages",
      "--output-dir",
      output_dir.path().to_str().unwrap(),
    ])
    .assert()
    .success();

  assert!(output_dir.path().join("gest.1").exists(), "root man page should exist");
}

#[test]
fn it_writes_subcommand_man_pages() {
  let env = GestCmd::new();
  let output_dir = tempfile::tempdir().expect("failed to create temp dir for man pages");

  env
    .run([
      "generate",
      "man-pages",
      "--output-dir",
      output_dir.path().to_str().unwrap(),
    ])
    .assert()
    .success();

  assert!(
    output_dir.path().join("gest-generate.1").exists(),
    "gest-generate man page should exist"
  );
  assert!(
    output_dir.path().join("gest-config.1").exists(),
    "gest-config man page should exist"
  );
}
