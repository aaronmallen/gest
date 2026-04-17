use crate::support::helpers::GestCmd;

#[test]
fn it_accepts_positional_rel_for_iteration_link_with_deprecation_warning() {
  let g = GestCmd::new();
  let source = g.create_iteration("legacy-src");
  let target = g.create_iteration("legacy-dst");

  let output = g
    .cmd()
    .args(["iteration", "link", &source, "blocks", &target])
    .output()
    .expect("iteration link failed");

  assert!(
    output.status.success(),
    "legacy iteration link form should still succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("passing <rel> as a positional argument is deprecated"),
    "stderr should contain deprecation warning, got: {stderr}"
  );
  assert!(
    stderr.contains("use --rel <type>"),
    "stderr should suggest using --rel, got: {stderr}"
  );
}

#[test]
fn it_accepts_rel_flag_for_iteration_link() {
  let g = GestCmd::new();
  let source = g.create_iteration("blocker-sprint");
  let target = g.create_iteration("blocked-sprint");

  let output = g
    .cmd()
    .args(["iteration", "link", &source, &target, "--rel", "blocks"])
    .output()
    .expect("iteration link failed");

  assert!(
    output.status.success(),
    "iteration link with --rel should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let show = g
    .cmd()
    .args(["iteration", "show", &source])
    .output()
    .expect("iteration show failed");
  assert!(show.status.success(), "iteration show should succeed");
}

#[test]
fn it_errors_when_both_positional_rel_and_rel_flag_are_given() {
  let g = GestCmd::new();
  let source = g.create_iteration("conflict-src");
  let target = g.create_iteration("conflict-dst");

  let output = g
    .cmd()
    .args(["iteration", "link", &source, "blocks", &target, "--rel", "relates-to"])
    .output()
    .expect("iteration link failed");

  assert!(
    !output.status.success(),
    "iteration link should fail when both positional rel and --rel are provided"
  );

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("cannot specify rel both as a positional argument and via --rel"),
    "stderr should contain conflict error, got: {stderr}"
  );
}

#[test]
fn it_links_iteration_to_artifact() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("link-sprint");
  let artifact_id = g.create_artifact("Linked spec", "body");

  let output = g
    .cmd()
    .args(["iteration", "link", &iter_id, "relates-to", &artifact_id, "--artifact"])
    .output()
    .expect("iteration link failed");

  assert!(
    output.status.success(),
    "iteration link should succeed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
