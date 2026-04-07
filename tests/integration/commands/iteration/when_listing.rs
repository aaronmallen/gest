use crate::support::helpers::GestCmd;

#[test]
fn it_aliases_list_to_ls() {
  let g = GestCmd::new();
  g.create_iteration("Aliased listable");

  let output = g
    .cmd()
    .args(["iteration", "ls"])
    .output()
    .expect("iteration ls failed to run");

  assert!(output.status.success(), "iteration ls (alias) should succeed");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Aliased listable"), "got: {stdout}");
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
