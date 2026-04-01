use predicates::prelude::*;

use crate::support::helpers::GestCmd;

#[test]
fn it_creates_an_artifact() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "create", "--title", "My spec", "--body", "spec content"])
    .assert()
    .success()
    .stdout(predicate::str::contains("created artifact"));
}

#[test]
fn it_creates_with_type_adr() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "ADR Artifact",
      "--body",
      "adr body",
      "--type",
      "adr",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created artifact"));
}

#[test]
fn it_creates_with_type_rfc() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "RFC Artifact",
      "--body",
      "rfc body",
      "--type",
      "rfc",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created artifact"));
}

#[test]
fn it_creates_with_type_spec() {
  let env = GestCmd::new();

  env
    .cmd()
    .args([
      "artifact",
      "create",
      "--title",
      "Spec Artifact",
      "--body",
      "spec body",
      "--type",
      "spec",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("created artifact"));
}

#[test]
fn it_errors_without_title() {
  let env = GestCmd::new();

  env
    .cmd()
    .args(["artifact", "create", "--body", "body without a heading"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("No title found"));
}
