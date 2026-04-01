use crate::support::helpers::GestCmd;

fn create_iteration(env: &GestCmd, title: &str) -> String {
  let output = env
    .cmd()
    .args(["iteration", "create", title])
    .output()
    .expect("failed to run gest iteration create");

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .split_whitespace()
    .last()
    .expect("no output from iteration create")
    .to_string()
}

#[test]
fn it_shows_iteration_status() {
  let env = GestCmd::new();
  let iter_id = create_iteration(&env, "Sprint 1");

  env.cmd().args(["iteration", "status", &iter_id]).assert().success();
}
