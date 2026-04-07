use crate::support::helpers::GestCmd;

#[test]
fn it_shows_the_resolved_config() {
  let g = GestCmd::new();
  let output = g
    .cmd()
    .args(["config", "show"])
    .output()
    .expect("config show failed to run");

  assert!(output.status.success(), "config show exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("[log]"), "got: {stdout}");
}
