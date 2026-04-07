use crate::support::helpers::GestCmd;

#[test]
fn it_finds_a_task_by_title() {
  let g = GestCmd::new();
  g.create_task("Searchable unicorn task");

  let output = g
    .cmd()
    .args(["search", "unicorn"])
    .output()
    .expect("search failed to run");

  assert!(output.status.success(), "search exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Searchable unicorn task"), "got: {stdout}");
}

#[test]
fn it_returns_no_results_for_unmatched_query() {
  let g = GestCmd::new();
  g.create_task("Regular task");

  let output = g
    .cmd()
    .args(["search", "zyxwvuts"])
    .output()
    .expect("search failed to run");

  assert!(output.status.success(), "search exited non-zero");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("no results"), "got: {stdout}");
}
