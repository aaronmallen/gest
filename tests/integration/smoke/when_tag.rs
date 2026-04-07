use crate::support::helpers::GestCmd;

fn attach_tag(g: &GestCmd, entity: &str, id: &str, label: &str) {
  let output = g
    .cmd()
    .args([entity, "tag", id, label])
    .output()
    .expect("tag failed to run");
  assert!(
    output.status.success(),
    "{entity} tag exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

fn list_tags(g: &GestCmd, extra: &[&str]) -> String {
  let mut args = vec!["tag", "list"];
  args.extend_from_slice(extra);
  let output = g.cmd().args(&args).output().expect("tag list failed to run");
  assert!(
    output.status.success(),
    "tag list exited non-zero: {}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn it_lists_all_tags_without_filter() {
  let g = GestCmd::new();
  let task_id = g.create_task("Tagged task");
  let artifact_id = g.create_artifact("Tagged artifact", "body");

  attach_tag(&g, "task", &task_id, "task-tag");
  attach_tag(&g, "artifact", &artifact_id, "artifact-tag");

  let stdout = list_tags(&g, &[]);
  assert!(stdout.contains("task-tag"), "got: {stdout}");
  assert!(stdout.contains("artifact-tag"), "got: {stdout}");
}

#[test]
fn it_filters_to_task_tags_only() {
  let g = GestCmd::new();
  let task_id = g.create_task("Tagged task");
  let artifact_id = g.create_artifact("Tagged artifact", "body");

  attach_tag(&g, "task", &task_id, "only-on-task");
  attach_tag(&g, "artifact", &artifact_id, "only-on-artifact");

  let stdout = list_tags(&g, &["--task"]);
  assert!(stdout.contains("only-on-task"), "got: {stdout}");
  assert!(!stdout.contains("only-on-artifact"), "got: {stdout}");
}

#[test]
fn it_filters_to_artifact_tags_only() {
  let g = GestCmd::new();
  let task_id = g.create_task("Tagged task");
  let artifact_id = g.create_artifact("Tagged artifact", "body");

  attach_tag(&g, "task", &task_id, "only-on-task");
  attach_tag(&g, "artifact", &artifact_id, "only-on-artifact");

  let stdout = list_tags(&g, &["--artifact"]);
  assert!(stdout.contains("only-on-artifact"), "got: {stdout}");
  assert!(!stdout.contains("only-on-task"), "got: {stdout}");
}

#[test]
fn it_filters_to_iteration_tags_only() {
  let g = GestCmd::new();
  let task_id = g.create_task("Tagged task");
  let iteration_id = g.create_iteration("Tagged iteration");

  attach_tag(&g, "task", &task_id, "only-on-task");
  attach_tag(&g, "iteration", &iteration_id, "only-on-iteration");

  let stdout = list_tags(&g, &["--iteration"]);
  assert!(stdout.contains("only-on-iteration"), "got: {stdout}");
  assert!(!stdout.contains("only-on-task"), "got: {stdout}");
}

#[test]
fn it_rejects_combined_filters() {
  let g = GestCmd::new();

  let output = g
    .cmd()
    .args(["tag", "list", "--task", "--artifact"])
    .output()
    .expect("tag list failed to run");

  assert!(
    !output.status.success(),
    "tag list should reject combined filters, got success"
  );
}
