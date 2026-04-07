use crate::support::helpers::GestCmd;

#[test]
fn it_prints_the_version_banner() {
  let g = GestCmd::new_uninit();
  let output = g.cmd().args(["--version"]).output().expect("--version failed to run");

  let combined = format!(
    "{}{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  );
  // The version line embeds the crate version, which starts with "v".
  assert!(
    combined.contains(env!("CARGO_PKG_VERSION")),
    "version output should contain CARGO_PKG_VERSION, got: {combined}"
  );
}
