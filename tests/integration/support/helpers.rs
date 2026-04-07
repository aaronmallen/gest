use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

/// Test helper wrapping `assert_cmd::Command` with per-test filesystem isolation.
pub struct GestCmd {
  temp_dir: TempDir,
}

impl GestCmd {
  fn build_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("gest").expect("gest binary not found");
    let path = temp_dir.path();
    cmd.current_dir(path);
    cmd.env("GEST_CONFIG", path.join("gest.toml"));
    cmd.env("GEST_STORAGE__DATA_DIR", path.join(".gest-data"));
    cmd.env("GEST_PROJECT_DIR", path.join(".gest"));
    cmd.env("GEST_STATE_DIR", path.join(".gest-state"));
    cmd
  }

  /// Create a new `GestCmd` with a fresh temp directory and run `gest init` in it.
  pub fn new() -> Self {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // Pre-create the data and state directories so config resolution succeeds
    // before `init` populates the full directory tree.
    std::fs::create_dir_all(temp_dir.path().join(".gest")).expect("failed to create .gest dir");
    std::fs::create_dir_all(temp_dir.path().join(".gest-data")).expect("failed to create .gest-data dir");
    std::fs::create_dir_all(temp_dir.path().join(".gest-state")).expect("failed to create .gest-state dir");

    // Initialize a gest project in the temp dir
    let mut init = Self::build_cmd(&temp_dir);
    init.args(["init"]);
    init.assert().success();

    Self {
      temp_dir,
    }
  }

  /// Create a new `GestCmd` with a fresh temp directory without running `gest init`.
  ///
  /// The data and state directories are created so that config loading succeeds,
  /// but the store structure is not initialized. Useful for commands that do not
  /// need an initialized project (e.g. `version`).
  pub fn new_uninit() -> Self {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    std::fs::create_dir_all(temp_dir.path().join(".gest")).expect("failed to create .gest dir");
    std::fs::create_dir_all(temp_dir.path().join(".gest-data")).expect("failed to create .gest-data dir");
    std::fs::create_dir_all(temp_dir.path().join(".gest-state")).expect("failed to create .gest-state dir");

    Self {
      temp_dir,
    }
  }

  /// Add a tag to any entity via the top-level `gest tag add` command.
  pub fn add_tag_direct(&self, entity_id: &str, label: &str) {
    let output = self
      .cmd()
      .args(["tag", "add", entity_id, label])
      .output()
      .expect("tag add failed to run");
    assert!(
      output.status.success(),
      "tag add exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Advance an iteration to its next phase.
  pub fn advance_iteration(&self, id: &str) {
    let output = self
      .cmd()
      .args(["iteration", "advance", id])
      .output()
      .expect("iteration advance failed to run");
    assert!(
      output.status.success(),
      "iteration advance exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Attach a tag to an entity via the entity-specific `<entity> tag` subcommand
  /// (e.g. `gest task tag <id> <label>`).
  pub fn attach_tag(&self, entity: &str, id: &str, label: &str) {
    let output = self
      .cmd()
      .args([entity, "tag", id, label])
      .output()
      .expect("entity tag failed to run");
    assert!(
      output.status.success(),
      "{entity} tag exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Mark `blocker` as blocking `blocked` via `gest task block`.
  pub fn block_task(&self, blocker: &str, blocked: &str) {
    let output = self
      .cmd()
      .args(["task", "block", blocker, blocked])
      .output()
      .expect("task block failed to run");
    assert!(
      output.status.success(),
      "task block exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Cancel an iteration.
  pub fn cancel_iteration(&self, id: &str) {
    let output = self
      .cmd()
      .args(["iteration", "cancel", id])
      .output()
      .expect("iteration cancel failed to run");
    assert!(
      output.status.success(),
      "iteration cancel exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Cancel a task.
  pub fn cancel_task(&self, id: &str) {
    let output = self
      .cmd()
      .args(["task", "cancel", id])
      .output()
      .expect("task cancel failed to run");
    assert!(
      output.status.success(),
      "task cancel exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Claim a task (assign and mark in-progress).
  pub fn claim_task(&self, id: &str) {
    let output = self
      .cmd()
      .args(["task", "claim", id])
      .output()
      .expect("task claim failed to run");
    assert!(
      output.status.success(),
      "task claim exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Return a `Command` pre-configured with isolation env vars and color disabled.
  pub fn cmd(&self) -> Command {
    let mut cmd = Self::build_cmd(&self.temp_dir);
    cmd.env("NO_COLOR", "1");
    cmd
  }

  /// Complete an iteration.
  pub fn complete_iteration(&self, id: &str) {
    let output = self
      .cmd()
      .args(["iteration", "complete", id])
      .output()
      .expect("iteration complete failed to run");
    assert!(
      output.status.success(),
      "iteration complete exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Mark a task as done via `gest task complete`.
  pub fn complete_task(&self, id: &str) {
    let output = self
      .cmd()
      .args(["task", "complete", id])
      .output()
      .expect("task complete failed to run");
    assert!(
      output.status.success(),
      "task complete exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Create an artifact with the given title and body, returning the new artifact ID.
  pub fn create_artifact(&self, title: &str, body: &str) -> String {
    let output = self
      .cmd()
      .args(["artifact", "create", title, "--body", body])
      .output()
      .expect("failed to run artifact create");

    assert!(
      output.status.success(),
      "artifact create failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_id_from_create_output(&stdout)
      .unwrap_or_else(|| panic!("could not extract artifact ID from output:\n{stdout}"))
  }

  /// Create an iteration with the given title, returning the new iteration ID.
  pub fn create_iteration(&self, title: &str) -> String {
    let output = self
      .cmd()
      .args(["iteration", "create", title])
      .output()
      .expect("failed to run iteration create");

    assert!(
      output.status.success(),
      "iteration create failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_id_from_create_output(&stdout)
      .unwrap_or_else(|| panic!("could not extract iteration ID from output:\n{stdout}"))
  }

  /// Create an iteration seeded with tasks in the given phases.
  ///
  /// `phases[i]` is the slice of task titles assigned to phase `i + 1`. Returns the
  /// iteration ID. Tasks are created fresh via `create_task` and attached with
  /// `gest iteration add --phase`.
  pub fn create_iteration_with_phases(&self, title: &str, phases: &[&[&str]]) -> String {
    let iter_id = self.create_iteration(title);
    for (i, titles) in phases.iter().enumerate() {
      let phase = (i + 1).to_string();
      for task_title in *titles {
        let task_id = self.create_task(task_title);
        let output = self
          .cmd()
          .args(["iteration", "add", &iter_id, &task_id, "--phase", &phase])
          .output()
          .expect("iteration add failed to run");
        assert!(
          output.status.success(),
          "iteration add exited non-zero: {}",
          String::from_utf8_lossy(&output.stderr)
        );
      }
    }
    iter_id
  }

  /// Create a task with the given title, returning the new task ID.
  pub fn create_task(&self, title: &str) -> String {
    let output = self
      .cmd()
      .args(["task", "create", title])
      .output()
      .expect("failed to run task create");

    assert!(
      output.status.success(),
      "task create failed: {}",
      String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_id_from_create_output(&stdout).unwrap_or_else(|| panic!("could not extract task ID from output:\n{stdout}"))
  }

  /// Initialize an additional project rooted at `dir` against the same data store as `self`.
  ///
  /// Used to seed multi-project scenarios. The caller is responsible for creating `dir`
  /// if it does not already exist under `temp_dir_path()`.
  pub fn init_extra_project(&self, dir: &Path) {
    std::fs::create_dir_all(dir).expect("failed to create extra project dir");
    let data_dir = self.temp_dir.path().join(".gest-data");
    let state_dir = self.temp_dir.path().join(".gest-state");
    let config = self.temp_dir.path().join("gest.toml");
    let project_dir = dir.join(".gest");
    std::fs::create_dir_all(&project_dir).expect("failed to create extra .gest dir");

    Command::cargo_bin("gest")
      .expect("gest binary not found")
      .current_dir(dir)
      .env("GEST_CONFIG", config)
      .env("GEST_STORAGE__DATA_DIR", data_dir)
      .env("GEST_PROJECT_DIR", project_dir)
      .env("GEST_STATE_DIR", state_dir)
      .env("NO_COLOR", "1")
      .args(["init"])
      .assert()
      .success();
  }

  /// Link a task to another entity via `gest task link`.
  ///
  /// `target_type` should be one of `task`, `artifact`, or `iteration`. `rel` sets the
  /// relationship type (e.g. `relates-to`, `blocks`).
  pub fn link_task(&self, task_id: &str, target_id: &str, target_type: &str, rel: &str) {
    let output = self
      .cmd()
      .args([
        "task",
        "link",
        task_id,
        target_id,
        "--target-type",
        target_type,
        "--rel",
        rel,
      ])
      .output()
      .expect("task link failed to run");
    assert!(
      output.status.success(),
      "task link exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Return a `Command` with isolation env vars but no extra args.
  pub fn raw_cmd(&self) -> Command {
    Self::build_cmd(&self.temp_dir)
  }

  /// Reopen a completed or cancelled iteration.
  pub fn reopen_iteration(&self, id: &str) {
    let output = self
      .cmd()
      .args(["iteration", "reopen", id])
      .output()
      .expect("iteration reopen failed to run");
    assert!(
      output.status.success(),
      "iteration reopen exited non-zero: {}",
      String::from_utf8_lossy(&output.stderr)
    );
  }

  /// Shorthand for `cmd().args(args).assert()`.
  pub fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
    self.cmd().args(args).assert()
  }

  /// Return the temp directory path.
  pub fn temp_dir_path(&self) -> &Path {
    self.temp_dir.path()
  }
}

/// Extract the entity ID from a "created <entity>  <id>" output line.
pub fn extract_id_from_create_output(output: &str) -> Option<String> {
  output
    .lines()
    .find(|line| line.to_lowercase().contains("created"))
    .and_then(|line| line.split_whitespace().last().map(str::to_string))
}

/// Extract the rendered prefix length for `short_id` from a colored output buffer.
///
/// IDs are displayed as `<CSI>...m{prefix}<CSI>0m<CSI>...m{rest}<CSI>0m`. We scan for the id
/// as a contiguous run of visible characters, allowing escape sequences to interleave; the
/// first interleaved escape after at least one visible character marks the prefix→rest
/// boundary.
pub fn rendered_prefix_len(output: &str, short_id: &str) -> Option<usize> {
  let bytes = output.as_bytes();
  let target = short_id.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let mut j = i;
    let mut t = 0;
    let mut prefix_len: Option<usize> = None;
    let mut visible_seen = 0usize;
    let mut last_was_visible = true;
    while t < target.len() && j < bytes.len() {
      if bytes[j] == 0x1b && j + 1 < bytes.len() && bytes[j + 1] == b'[' {
        if t > 0 && prefix_len.is_none() && last_was_visible {
          prefix_len = Some(visible_seen);
        }
        j += 2;
        while j < bytes.len() && !(0x40..=0x7e).contains(&bytes[j]) {
          j += 1;
        }
        if j < bytes.len() {
          j += 1;
        }
        last_was_visible = false;
        continue;
      }
      if bytes[j] == target[t] {
        t += 1;
        j += 1;
        visible_seen += 1;
        last_was_visible = true;
      } else {
        break;
      }
    }
    if t == target.len() {
      return Some(prefix_len.unwrap_or(visible_seen));
    }
    i += 1;
  }
  None
}

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
      i += 2;
      while i < bytes.len() && !(0x40..=0x7e).contains(&bytes[i]) {
        i += 1;
      }
      if i < bytes.len() {
        i += 1;
      }
    } else {
      out.push(bytes[i] as char);
      i += 1;
    }
  }
  out
}
