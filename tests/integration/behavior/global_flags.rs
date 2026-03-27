use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_accepts_single_verbose_flag() {
  let env = GestCmd::new();

  env.cmd().args(["-v", "--help"]).assert().success();
}

#[test]
fn it_accepts_double_verbose_flag() {
  let env = GestCmd::new();

  env.cmd().args(["-vv", "--help"]).assert().success();
}

#[test]
fn it_accepts_triple_verbose_flag() {
  let env = GestCmd::new();

  env.cmd().args(["-vvv", "--help"]).assert().success();
}

#[test]
fn it_accepts_long_verbose_flag() {
  let env = GestCmd::new();

  env.cmd().args(["--verbose", "--help"]).assert().success();
}

#[test]
fn it_does_not_emit_ansi_codes_in_subprocess_stdout() {
  let env = GestCmd::new();
  let ansi_escape = predicate::str::is_match(r"\x1b\[").unwrap();

  env.cmd().arg("--help").assert().success().stdout(ansi_escape.not());
}
