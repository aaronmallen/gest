//! Unix domain socket listener that forwards bare connection signals to the SSE
//! reload broadcast channel.
//!
//! The protocol is intentionally trivial: any successful `connect()` is interpreted
//! as a "reload now" signal. No bytes are exchanged. The CLI side
//! ([`crate::cli::web_notify`]) opens and immediately drops a `UnixStream` to fire
//! the signal.

#[cfg(unix)]
use std::path::{Path, PathBuf};

#[cfg(unix)]
use tokio::{net::UnixListener, sync::broadcast::Sender, task::JoinHandle};

/// Bind a unix domain socket listener at `path`, transparently recovering from a
/// stale socket file left behind by a previous process.
///
/// On `EADDRINUSE` the function probes the existing socket with a sync `connect()`.
/// If something is actually listening the original error is returned (the caller
/// should not steal the socket from a live process). Otherwise the stale file is
/// unlinked and the bind is retried once.
#[cfg(unix)]
pub fn bind_reload_socket(path: &Path) -> std::io::Result<UnixListener> {
  use std::{io::ErrorKind, os::unix::net::UnixStream as StdUnixStream};

  match UnixListener::bind(path) {
    Ok(listener) => Ok(listener),
    Err(err) if err.kind() == ErrorKind::AddrInUse => {
      if StdUnixStream::connect(path).is_ok() {
        return Err(err);
      }
      std::fs::remove_file(path)?;
      UnixListener::bind(path)
    }
    Err(err) => Err(err),
  }
}

/// Spawn an accept loop that fires a reload signal on every incoming connection.
///
/// The streams are dropped immediately — the bare connection itself is the entire
/// payload. The returned [`JoinHandle`] is detached by callers in production and
/// joined in tests; the runtime aborts the task when the server's tokio runtime
/// shuts down.
#[cfg(unix)]
pub fn spawn_reload_listener(listener: UnixListener, tx: Sender<()>) -> JoinHandle<()> {
  tokio::spawn(async move {
    loop {
      match listener.accept().await {
        Ok((_stream, _addr)) => {
          let _ = tx.send(());
        }
        Err(err) => {
          log::warn!("reload socket accept error: {err}");
        }
      }
    }
  })
}

/// Drop guard that removes the reload socket file when the web server exits cleanly.
///
/// Abrupt termination (SIGKILL, panic during runtime shutdown) may leave the file
/// behind; [`bind_reload_socket`] handles that case on the next startup.
#[cfg(unix)]
pub struct ReloadSocketGuard {
  path: PathBuf,
}

#[cfg(unix)]
impl ReloadSocketGuard {
  /// Create a guard that will unlink `path` when dropped.
  pub fn new(path: PathBuf) -> Self {
    Self {
      path,
    }
  }
}

#[cfg(unix)]
impl Drop for ReloadSocketGuard {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.path);
  }
}

#[cfg(all(test, unix))]
mod tests {
  use super::*;

  mod bind_reload_socket {
    use super::*;

    #[tokio::test]
    async fn it_binds_when_path_is_free() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("web.sock");

      let listener = bind_reload_socket(&path).unwrap();

      assert!(path.exists());
      drop(listener);
    }

    #[tokio::test]
    async fn it_recovers_from_a_stale_socket_file() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("web.sock");
      // Simulate a stale socket file: a regular file at the bind path.
      std::fs::write(&path, b"stale").unwrap();

      let listener = bind_reload_socket(&path).unwrap();

      assert!(path.exists());
      drop(listener);
    }

    #[tokio::test]
    async fn it_refuses_to_steal_a_live_socket() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("web.sock");
      let _live = bind_reload_socket(&path).unwrap();

      let result = bind_reload_socket(&path);

      assert!(result.is_err());
    }
  }

  mod reload_socket_guard {
    use super::*;

    #[test]
    fn it_removes_the_socket_file_on_drop() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("web.sock");
      std::fs::write(&path, b"").unwrap();
      let guard = ReloadSocketGuard::new(path.clone());

      drop(guard);

      assert!(!path.exists());
    }

    #[test]
    fn it_tolerates_a_missing_file_on_drop() {
      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("missing.sock");
      let guard = ReloadSocketGuard::new(path.clone());

      drop(guard);

      assert!(!path.exists());
    }
  }

  mod spawn_reload_listener {
    use super::*;

    #[tokio::test]
    async fn it_forwards_connections_as_reload_signals() {
      use tokio::net::UnixStream;

      let tmp = tempfile::tempdir().unwrap();
      let path = tmp.path().join("web.sock");
      let listener = bind_reload_socket(&path).unwrap();
      let (tx, mut rx) = tokio::sync::broadcast::channel::<()>(4);
      let handle = spawn_reload_listener(listener, tx);

      let _client = UnixStream::connect(&path).await.unwrap();
      let recv = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .expect("reload signal should arrive");

      assert!(recv.is_ok());
      handle.abort();
    }
  }
}
