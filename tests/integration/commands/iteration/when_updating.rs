use crate::support::helpers::GestCmd;

#[test]
fn it_aliases_update_to_edit() {
  let g = GestCmd::new();
  let id = g.create_iteration("editable sprint");

  let output = g
    .cmd()
    .args(["iteration", "edit", &id, "--title", "renamed sprint"])
    .output()
    .expect("iteration edit failed to run");

  assert!(
    output.status.success(),
    "iteration edit (alias) should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let show = g
    .cmd()
    .args(["iteration", "show", &id])
    .output()
    .expect("iteration show failed");
  let stdout = String::from_utf8_lossy(&show.stdout);
  assert!(stdout.contains("renamed sprint"), "got: {stdout}");
}
