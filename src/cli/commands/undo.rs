use std::fs;

use chrono::Utc;
use clap::Args;

use crate::{
  cli::{self, AppContext, capture},
  event_store::{EventStore, EventType, Transaction},
  ui::composites::success_message::SuccessMessage,
};

/// Undo the most recent mutating command(s).
#[derive(Debug, Args)]
pub struct Command {
  /// Number of commands to undo (default: 1).
  #[arg(default_value = "1")]
  steps: usize,
}

impl Command {
  /// Reverse the N most recent non-undone transactions by restoring file snapshots.
  pub fn call(&self, ctx: &AppContext) -> cli::Result<()> {
    let store = EventStore::open(ctx.settings.state_dir())
      .map_err(|e| cli::Error::generic(format!("failed to open event store: {e}")))?;

    let project_id = capture::project_id(&ctx.settings);
    let data_dir = ctx.settings.data_dir();

    let mut undone = 0;
    for _ in 0..self.steps {
      let tx = store
        .latest_undoable(&project_id)
        .map_err(|e| cli::Error::generic(format!("failed to query event store: {e}")))?;

      let Some(tx) = tx else {
        break;
      };

      apply_undo(data_dir, &tx)?;

      store
        .mark_undone(&tx.id)
        .map_err(|e| cli::Error::generic(format!("failed to mark transaction undone: {e}")))?;

      let file_count = tx.events.len();
      let files_word = if file_count == 1 { "file" } else { "files" };
      let ago = format_elapsed(tx.created_at);
      let msg = format!("Undid '{}' ({file_count} {files_word}, {ago})", tx.command);
      println!("{}", SuccessMessage::new(&msg, &ctx.theme));

      undone += 1;
    }

    if undone == 0 {
      return Err(cli::Error::no_result("Nothing to undo"));
    }

    Ok(())
  }
}

/// Reverse a single transaction by restoring file snapshots.
fn apply_undo(data_dir: &std::path::Path, tx: &Transaction) -> cli::Result<()> {
  for event in &tx.events {
    let file_path = data_dir.join(&event.file_path);
    match event.event_type {
      EventType::Created => {
        // Undo a create → delete the file.
        if file_path.exists() {
          fs::remove_file(&file_path)?;
        }
      }
      EventType::Modified => {
        // Undo a modify → restore the prior content.
        if let Some(content) = &event.before_content {
          fs::write(&file_path, content)?;
        }
      }
      EventType::Deleted => {
        // Undo a delete → restore the file.
        if let Some(content) = &event.before_content {
          if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
          }
          fs::write(&file_path, content)?;
        }
      }
    }
  }
  Ok(())
}

/// Format the elapsed time since a timestamp as a human-readable string.
fn format_elapsed(created_at: chrono::DateTime<Utc>) -> String {
  let elapsed = Utc::now().signed_duration_since(created_at);
  let secs = elapsed.num_seconds();
  if secs < 60 {
    format!("{secs}s ago")
  } else if secs < 3600 {
    format!("{}m ago", secs / 60)
  } else if secs < 86400 {
    format!("{}h ago", secs / 3600)
  } else {
    format!("{}d ago", secs / 86400)
  }
}
