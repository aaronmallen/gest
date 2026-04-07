//! Tests for global CLI flags that apply across every command: `--help`, `--version`,
//! and error behavior for unknown flags/subcommands.

use crate::support::helpers::GestCmd;

#[test]
fn it_prints_help_with_long_flag() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["--help"]).output().expect("--help failed to run");

  assert!(output.status.success(), "--help should exit zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Usage:"),
    "help output should contain Usage, got: {stdout}"
  );

  // Each top-level subcommand should also expose --help.
  for sub in &[
    "artifact",
    "config",
    "generate",
    "init",
    "iteration",
    "project",
    "search",
    "tag",
    "task",
    "undo",
  ] {
    let out = g
      .cmd()
      .args([sub, "--help"])
      .output()
      .unwrap_or_else(|_| panic!("{sub} --help failed to run"));
    assert!(out.status.success(), "{sub} --help should exit zero");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Usage:"), "{sub} help should contain Usage, got: {text}");
  }
}

#[test]
fn it_prints_version_with_long_flag() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["--version"]).output().expect("--version failed to run");

  assert!(output.status.success(), "--version should exit zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains(env!("CARGO_PKG_VERSION")),
    "--version output should contain crate version, got: {stdout}"
  );
}

#[test]
fn it_prints_version_with_short_flag() {
  let g = GestCmd::new_uninit();

  let output = g.cmd().args(["-V"]).output().expect("-V failed to run");

  assert!(output.status.success(), "-V should exit zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains(env!("CARGO_PKG_VERSION")),
    "-V output should contain crate version, got: {stdout}"
  );
}

#[test]
fn it_rejects_unknown_flag() {
  let g = GestCmd::new_uninit();

  let output = g
    .cmd()
    .args(["--definitely-not-a-flag"])
    .output()
    .expect("unknown flag failed to run");

  assert!(!output.status.success(), "unknown flag should exit non-zero");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.to_lowercase().contains("unexpected") || stderr.to_lowercase().contains("unknown"),
    "stderr should mention unknown/unexpected argument, got: {stderr}"
  );
}

#[test]
fn it_rejects_unknown_subcommand() {
  let g = GestCmd::new_uninit();

  let output = g
    .cmd()
    .args(["definitely-not-a-subcommand"])
    .output()
    .expect("unknown subcommand failed to run");

  assert!(!output.status.success(), "unknown subcommand should exit non-zero");
}
