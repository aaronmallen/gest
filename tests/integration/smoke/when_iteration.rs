use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_iteration() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["iteration", "create", "Sprint 1"])
    .output()
    .expect("iteration create failed to run");

  assert!(output.status.success(), "iteration create exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("created iteration"), "got: {stdout}");
  assert!(stdout.contains("Sprint 1"), "got: {stdout}");
}

#[test]
fn it_lists_iterations() {
  let g = GestCmd::new();
  g.create_iteration("Listable sprint");

  let output = g
    .cmd()
    .args(["iteration", "list"])
    .output()
    .expect("iteration list failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Listable sprint"), "got: {stdout}");
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
