use std::path::{Path, PathBuf};

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
    std::fs::create_dir_all(temp_dir.path().join(".gest-state")).expect("failed to create .gest-state dir");

    Self {
      temp_dir,
    }
  }

  /// Return a `Command` pre-configured with isolation env vars and color disabled.
  pub fn cmd(&self) -> Command {
    let mut cmd = Self::build_cmd(&self.temp_dir);
    cmd.env("NO_COLOR", "1");
    cmd
  }

  /// Create an artifact with the given title and body, returning the new artifact ID.
  pub fn create_artifact(&self, title: &str, body: &str) -> String {
    let output = self
      .cmd()
      .args(["artifact", "create", "--title", title, "--body", body])
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

  fn data_dir(&self) -> PathBuf {
    self.temp_dir.path().join(".gest")
  }

  /// Return a `Command` with isolation env vars but no extra args.
  pub fn raw_cmd(&self) -> Command {
    Self::build_cmd(&self.temp_dir)
  }

  /// Read a file relative to the data directory.
  pub fn read_data_file(&self, relative: &str) -> String {
    let path = self.data_dir().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
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
    .find(|line| line.contains("created"))
    .and_then(|line| line.split_whitespace().last().map(str::to_string))
}
