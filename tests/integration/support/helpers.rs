use std::{ffi::OsStr, fs, path::PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// Minimal TOML config for isolated tests.
const TEST_CONFIG: &str = r#"
[log]

[storage]
"#;

/// Builder for running the `gest` binary in an isolated temp environment.
///
/// Each instance creates its own temp directory with a `.gest/` structure and
/// config file, ensuring complete test isolation.
pub struct GestCmd {
  config_path: PathBuf,
  _temp_dir: TempDir,
}

#[allow(dead_code)]
impl GestCmd {
  /// Create a new isolated test environment.
  ///
  /// Runs `gest init` in a fresh temp directory to set up the `.gest/`
  /// structure, and writes a minimal config file for isolation.
  pub fn new() -> Self {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, TEST_CONFIG).expect("failed to write test config");

    // Run `gest init` to create the .gest/ directory structure.
    Command::cargo_bin("gest")
      .expect("failed to find gest binary")
      .arg("init")
      .current_dir(temp_dir.path())
      .env("GEST_CONFIG", &config_path)
      .env("GEST_DATA_DIR", temp_dir.path().join(".gest"))
      .env_remove("EDITOR")
      .env_remove("VISUAL")
      .assert()
      .success();

    Self {
      config_path,
      _temp_dir: temp_dir,
    }
  }

  /// Build an `assert_cmd::Command` for the `gest` binary with isolation env vars set.
  pub fn cmd(&self) -> Command {
    let mut cmd = Command::cargo_bin("gest").expect("failed to find gest binary");
    cmd.current_dir(self._temp_dir.path());
    cmd.env("GEST_CONFIG", &self.config_path);
    cmd.env("GEST_DATA_DIR", self._temp_dir.path().join(".gest"));
    cmd.env_remove("EDITOR");
    cmd.env_remove("VISUAL");
    cmd
  }

  /// Build an `assert_cmd::Command` with only env-var isolation (no extra default flags).
  ///
  /// Use this when testing global flags that conflict with the defaults added by [`cmd`].
  pub fn raw_cmd(&self) -> Command {
    let mut cmd = Command::cargo_bin("gest").expect("failed to find gest binary");
    cmd.current_dir(self._temp_dir.path());
    cmd.env("GEST_CONFIG", &self.config_path);
    cmd.env("GEST_DATA_DIR", self._temp_dir.path().join(".gest"));
    cmd
  }

  /// Read a file from the `.gest/` directory within the temp environment.
  pub fn read_data_file(&self, relative_path: &str) -> String {
    let path = self._temp_dir.path().join(".gest").join(relative_path);
    fs::read_to_string(path).unwrap_or_default()
  }

  /// Run a gest subcommand with the given arguments and return the command for assertions.
  pub fn run<I, S>(&self, args: I) -> Command
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let mut cmd = self.cmd();
    cmd.args(args);
    cmd
  }

  /// Return the path to the temporary directory backing this test environment.
  pub fn temp_dir_path(&self) -> &std::path::Path {
    self._temp_dir.path()
  }
}
