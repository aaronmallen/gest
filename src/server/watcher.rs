//! File watcher that monitors task and artifact directories for changes.
//!
//! Events are debounced so that rapid bursts of file operations (e.g. batch
//! task creation) collapse into a single notification on the provided
//! [`broadcast::Sender`].

use std::path::PathBuf;
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc;

use crate::config::Settings;

/// Default debounce window for collapsing rapid file-system events.
const DEBOUNCE: Duration = Duration::from_millis(200);

/// Returns `true` if the event should be ignored (e.g. atomic-write temp files).
fn is_filtered(event: &Event) -> bool {
  event.paths.iter().all(|p| {
    p.file_name()
      .and_then(|n| n.to_str())
      .is_some_and(|n| n.starts_with(".tmp_"))
  })
}

/// Collect the directories we need to watch.
///
/// For each entity directory we watch both the root and the `resolved/`
/// subdirectory (if it exists).
fn watched_dirs(settings: &Settings) -> Vec<PathBuf> {
  let roots = [settings.storage().task_dir(), settings.storage().artifact_dir()];

  let mut dirs = Vec::new();
  for root in roots {
    dirs.push(root.to_path_buf());
    let resolved = root.join("resolved");
    if resolved.is_dir() {
      dirs.push(resolved);
    }
  }
  dirs
}

/// Spawn a background tokio task that watches the file system and sends `()`
/// on `tx` whenever a relevant change is detected.
///
/// If none of the watched directories exist the function logs a warning and
/// returns without spawning a watcher.
pub fn spawn(settings: &Settings, tx: Sender<()>) -> tokio::task::JoinHandle<()> {
  let dirs = watched_dirs(settings);

  tokio::task::spawn(async move {
    // Channel used to bridge the synchronous notify callback into async land.
    let (notify_tx, mut notify_rx) = mpsc::channel::<()>(64);

    let watcher = {
      let notify_tx = notify_tx.clone();
      RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
          if let Ok(event) = res {
            if !is_filtered(&event) {
              // Non-blocking send — drop if the buffer is full.
              let _ = notify_tx.try_send(());
            }
          }
        },
        notify::Config::default(),
      )
    };

    let mut watcher = match watcher {
      Ok(w) => w,
      Err(e) => {
        log::warn!("Failed to create file watcher: {e}");
        return;
      }
    };

    let mut watching_any = false;
    for dir in &dirs {
      if !dir.exists() {
        log::warn!("Watch directory does not exist, skipping: {}", dir.display());
        continue;
      }
      if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
        log::warn!("Failed to watch {}: {e}", dir.display());
      } else {
        watching_any = true;
      }
    }

    if !watching_any {
      log::warn!("No directories to watch — file watcher exiting");
      return;
    }

    // Debounce loop: drain all pending events then wait for the debounce
    // window to elapse before sending a single notification.
    loop {
      // Wait for the first event.
      if notify_rx.recv().await.is_none() {
        break;
      }

      // Drain any events that arrive within the debounce window.
      tokio::time::sleep(DEBOUNCE).await;
      while notify_rx.try_recv().is_ok() {}

      // Broadcast the change. Ignore errors (no active receivers).
      let _ = tx.send(());
    }
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use notify::event::{CreateKind, EventKind};

  #[test]
  fn filters_tmp_files() {
    let event = Event {
      kind: EventKind::Create(CreateKind::File),
      paths: vec![PathBuf::from("/tasks/.tmp_12345")],
      attrs: Default::default(),
    };
    assert!(is_filtered(&event));
  }

  #[test]
  fn does_not_filter_normal_files() {
    let event = Event {
      kind: EventKind::Create(CreateKind::File),
      paths: vec![PathBuf::from("/tasks/abc123.toml")],
      attrs: Default::default(),
    };
    assert!(!is_filtered(&event));
  }

  #[test]
  fn mixed_paths_not_filtered() {
    let event = Event {
      kind: EventKind::Create(CreateKind::File),
      paths: vec![
        PathBuf::from("/tasks/.tmp_12345"),
        PathBuf::from("/tasks/abc123.toml"),
      ],
      attrs: Default::default(),
    };
    assert!(!is_filtered(&event));
  }
}
