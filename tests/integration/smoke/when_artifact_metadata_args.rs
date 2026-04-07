use crate::support::helpers::GestCmd;

fn read_meta(g: &GestCmd, id: &str, path: &str) -> String {
  let output = g
    .cmd()
    .args(["artifact", "meta", "get", id, path, "--raw"])
    .output()
    .unwrap();
  assert!(output.status.success(), "meta get failed");
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn it_creates_artifact_with_scalar_pairs() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args([
      "artifact",
      "create",
      "Title",
      "--body",
      "body",
      "-m",
      "count=42",
      "-m",
      "name=alice",
    ])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = crate::support::helpers::extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout))
    .expect("artifact id");

  assert_eq!(read_meta(&g, &id, "count"), "42");
  assert_eq!(read_meta(&g, &id, "name"), "alice");
}

#[test]
fn it_creates_artifact_with_dot_path_keys() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args([
      "artifact",
      "create",
      "Title",
      "--body",
      "body",
      "--metadata",
      "outer.inner=deep",
    ])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = crate::support::helpers::extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout)).unwrap();

  assert_eq!(read_meta(&g, &id, "outer.inner"), "deep");
}

#[test]
fn it_creates_artifact_with_metadata_json_merge() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args([
      "artifact",
      "create",
      "Title",
      "--body",
      "body",
      "-m",
      "k=1",
      "--metadata-json",
      r#"{"k":2,"extra":true}"#,
    ])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = crate::support::helpers::extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout)).unwrap();

  assert_eq!(read_meta(&g, &id, "k"), "2");
  assert_eq!(read_meta(&g, &id, "extra"), "true");
}

#[test]
fn it_updates_artifact_preserving_unrelated_keys() {
  let g = GestCmd::new();
  let id = g.create_artifact("Title", "body");

  g.cmd()
    .args(["artifact", "meta", "set", &id, "keep", "yes"])
    .assert()
    .success();

  g.cmd()
    .args(["artifact", "update", &id, "-m", "added=new"])
    .assert()
    .success();

  assert_eq!(read_meta(&g, &id, "keep"), "yes");
  assert_eq!(read_meta(&g, &id, "added"), "new");
}
