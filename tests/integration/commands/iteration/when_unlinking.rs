use serde_json::Value;

use crate::support::helpers::GestCmd;

#[test]
fn it_is_reversible_via_undo_for_iteration_to_iteration() {
  let g = GestCmd::new();
  let source = g.create_iteration("undo-src");
  let target = g.create_iteration("undo-dst");

  let link = g
    .cmd()
    .args(["iteration", "link", &source, &target, "--rel", "blocks"])
    .output()
    .expect("iteration link failed");
  assert!(link.status.success());

  let unlink = g
    .cmd()
    .args(["iteration", "unlink", &source, &target, "--rel", "blocks"])
    .output()
    .expect("iteration unlink failed");
  assert!(unlink.status.success());

  let undo = g.cmd().args(["undo"]).output().expect("undo failed");
  assert!(
    undo.status.success(),
    "undo should succeed: {}",
    String::from_utf8_lossy(&undo.stderr)
  );

  let source_rels = relationships_for(&g, &source);
  assert!(
    source_rels
      .iter()
      .any(|r| r["rel_type"] == "blocks" && r["target_id"].as_str().map(|s| s.starts_with(&target)).unwrap_or(false)),
    "undo should restore the forward (blocks) edge, got {source_rels:?}"
  );

  let target_rels = relationships_for(&g, &target);
  assert!(
    target_rels.iter().any(
      |r| r["rel_type"] == "blocked-by" && r["target_id"].as_str().map(|s| s.starts_with(&source)).unwrap_or(false)
    ),
    "undo should restore the reciprocal (blocked-by) edge, got {target_rels:?}"
  );
}

#[test]
fn it_removes_an_iteration_to_artifact_relationship() {
  let g = GestCmd::new();
  let iter_id = g.create_iteration("artifact-unlink-sprint");
  let artifact_id = g.create_artifact("Linked spec", "body");

  let link = g
    .cmd()
    .args([
      "iteration",
      "link",
      &iter_id,
      &artifact_id,
      "--artifact",
      "--rel",
      "relates-to",
    ])
    .output()
    .expect("iteration link failed");
  assert!(
    link.status.success(),
    "iteration link should succeed: {}",
    String::from_utf8_lossy(&link.stderr)
  );

  let unlink = g
    .cmd()
    .args([
      "iteration",
      "unlink",
      &iter_id,
      &artifact_id,
      "--artifact",
      "--rel",
      "relates-to",
    ])
    .output()
    .expect("iteration unlink failed");
  assert!(
    unlink.status.success(),
    "iteration unlink should succeed: {}",
    String::from_utf8_lossy(&unlink.stderr)
  );

  let iter_rels = relationships_for(&g, &iter_id);
  assert!(
    iter_rels.is_empty(),
    "iteration should have no relationships after artifact unlink, got {iter_rels:?}"
  );
}

#[test]
fn it_removes_both_halves_of_an_iteration_to_iteration_relationship() {
  let g = GestCmd::new();
  let source = g.create_iteration("unlink-src");
  let target = g.create_iteration("unlink-dst");

  let link = g
    .cmd()
    .args(["iteration", "link", &source, &target, "--rel", "blocks"])
    .output()
    .expect("iteration link failed");
  assert!(
    link.status.success(),
    "iteration link should succeed: {}",
    String::from_utf8_lossy(&link.stderr)
  );

  let unlink = g
    .cmd()
    .args(["iteration", "unlink", &source, &target, "--rel", "blocks"])
    .output()
    .expect("iteration unlink failed");
  assert!(
    unlink.status.success(),
    "iteration unlink should succeed: {}",
    String::from_utf8_lossy(&unlink.stderr)
  );

  let source_rels = relationships_for(&g, &source);
  assert!(
    source_rels.is_empty(),
    "source iteration should have no relationships after unlink, got {source_rels:?}"
  );

  let target_rels = relationships_for(&g, &target);
  assert!(
    target_rels.is_empty(),
    "target iteration should have no reciprocal relationship after unlink, got {target_rels:?}"
  );
}

/// Query `iteration show --json` and return the relationships array for `id`.
fn relationships_for(g: &GestCmd, id: &str) -> Vec<Value> {
  let output = g
    .cmd()
    .args(["iteration", "show", id, "--json"])
    .output()
    .expect("iteration show --json failed");
  assert!(
    output.status.success(),
    "iteration show --json failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8_lossy(&output.stdout);
  let json: Value = serde_json::from_str(&stdout).expect("invalid JSON from iteration show --json");
  json["relationships"]
    .as_array()
    .expect("relationships should be an array")
    .clone()
}
