use crate::support::helpers::GestCmd;

#[test]
fn it_initializes_a_new_project() {
  let g = GestCmd::new_uninit();
  let output = g.cmd().args(["init"]).output().expect("init failed to run");

  assert!(output.status.success(), "init exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("initialized project"), "got: {stdout}");
}

#[test]
fn it_reports_project_root_after_init() {
  let g = GestCmd::new_uninit();
  let output = g.cmd().args(["init"]).output().expect("init failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("root"),
    "init output should mention root, got: {stdout}"
  );
}
