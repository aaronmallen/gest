use crate::support::helpers::{GestCmd, extract_id_from_create_output};

fn read_meta(g: &GestCmd, id: &str, path: &str) -> String {
  let output = g
    .cmd()
    .args(["task", "meta", "get", id, path, "--raw"])
    .output()
    .unwrap();
  assert!(output.status.success(), "meta get failed");
  String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn it_creates_task_with_scalar_pairs() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["task", "create", "Title", "-m", "count=42", "-m", "name=alice"])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout)).unwrap();

  assert_eq!(read_meta(&g, &id, "count"), "42");
  assert_eq!(read_meta(&g, &id, "name"), "alice");
}

#[test]
fn it_creates_task_with_dot_path_keys() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["task", "create", "Title", "--metadata", "outer.inner=deep"])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout)).unwrap();

  assert_eq!(read_meta(&g, &id, "outer.inner"), "deep");
}

#[test]
fn it_creates_task_with_metadata_json_merge() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args([
      "task",
      "create",
      "Title",
      "-m",
      "k=1",
      "--metadata-json",
      r#"{"k":2,"extra":true}"#,
    ])
    .output()
    .unwrap();
  assert!(output.status.success());
  let id = extract_id_from_create_output(&String::from_utf8_lossy(&output.stdout)).unwrap();

  assert_eq!(read_meta(&g, &id, "k"), "2");
  assert_eq!(read_meta(&g, &id, "extra"), "true");
}

#[test]
fn it_updates_task_preserving_unrelated_keys() {
  let g = GestCmd::new();
  let id = g.create_task("Title");

  g.cmd()
    .args(["task", "meta", "set", &id, "keep", "yes"])
    .assert()
    .success();

  g.cmd()
    .args(["task", "update", &id, "-m", "added=new"])
    .assert()
    .success();

  assert_eq!(read_meta(&g, &id, "keep"), "yes");
  assert_eq!(read_meta(&g, &id, "added"), "new");
}
