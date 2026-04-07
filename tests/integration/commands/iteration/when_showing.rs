use crate::support::helpers::GestCmd;

#[test]
fn it_aliases_show_to_view() {
  let g = GestCmd::new();
  let id = g.create_iteration("Viewable sprint");

  let output = g
    .cmd()
    .args(["iteration", "view", &id])
    .output()
    .expect("iteration view failed to run");

  assert!(output.status.success(), "iteration view (alias) should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Viewable sprint"), "got: {stdout}");
}

#[test]
fn it_shows_iteration_status_for_new_iteration() {
  let g = GestCmd::new();
  let id = g.create_iteration("Status sprint");

  let output = g
    .cmd()
    .args(["iteration", "status", &id])
    .output()
    .expect("iteration status failed to run");

  assert!(output.status.success(), "iteration status exited non-zero");
}
