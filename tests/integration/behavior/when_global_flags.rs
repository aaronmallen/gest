use crate::support::helpers::GestCmd;

#[test]
fn it_supports_json_output() {
  let env = GestCmd::new();

  let output = env
    .cmd()
    .args(["task", "list", "--json"])
    .assert()
    .success()
    .get_output()
    .stdout
    .clone();

  let text = String::from_utf8(output).expect("stdout is not valid UTF-8");
  let parsed: serde_json::Value = serde_json::from_str(&text).expect("stdout is not valid JSON");

  assert!(parsed.is_array(), "expected a JSON array but got: {parsed}",);
}

#[test]
fn it_supports_no_color() {
  let env = GestCmd::new();

  // The NO_COLOR env var is the standard way to suppress color output.
  env.cmd().env("NO_COLOR", "1").arg("--help").assert().success();
}
