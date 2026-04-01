//! SQLite-backed event store for mutation tracking and undo.
//!
//! Records file-level snapshots of every mutating CLI command, grouped by
//! transaction. The event store lives in `<state_dir>/events.db` and uses
//! WAL mode for safe concurrent access.

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

/// Errors that can occur during event store operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Rusqlite(#[from] rusqlite::Error),
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

/// Convenience alias for event store operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The type of file mutation recorded in an event.
#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
  /// A file that did not exist before the command.
  Created,
  /// A file that existed and was changed.
  Modified,
  /// A file that existed and was removed.
  Deleted,
}

impl EventType {
  fn as_str(&self) -> &'static str {
    match self {
      Self::Created => "created",
      Self::Modified => "modified",
      Self::Deleted => "deleted",
    }
  }

  fn from_str(s: &str) -> Self {
    match s {
      "created" => Self::Created,
      "modified" => Self::Modified,
      "deleted" => Self::Deleted,
      other => panic!("unknown event type: {other}"),
    }
  }
}

/// A single file mutation within a transaction.
#[derive(Clone, Debug)]
pub struct Event {
  pub file_path: String,
  pub event_type: EventType,
  pub before_content: Option<Vec<u8>>,
}

/// A group of events from a single CLI invocation.
#[derive(Clone, Debug)]
pub struct Transaction {
  pub id: String,
  pub command: String,
  pub created_at: DateTime<Utc>,
  pub events: Vec<Event>,
}

/// Handle to the event store database.
pub struct EventStore {
  conn: Connection,
}

impl EventStore {
  /// Open (or create) the event store database at `<state_dir>/events.db`.
  ///
  /// Creates the schema on first use and enables WAL mode.
  pub fn open(state_dir: &Path) -> Result<Self> {
    std::fs::create_dir_all(state_dir)?;
    let db_path = state_dir.join("events.db");
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "wal")?;
    conn.pragma_update(None, "foreign_keys", "on")?;
    create_schema(&conn)?;
    Ok(Self {
      conn,
    })
  }

  /// Open an in-memory event store (for testing).
  #[cfg(test)]
  pub fn open_in_memory() -> Result<Self> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "on")?;
    create_schema(&conn)?;
    Ok(Self {
      conn,
    })
  }

  /// Begin a new transaction for a CLI command invocation.
  ///
  /// Returns the transaction ID (a UUID-like random hex string).
  pub fn begin_transaction(&self, project_id: &str, command: &str) -> Result<String> {
    let id = generate_id();
    let now = Utc::now().to_rfc3339();
    self.conn.execute(
      "INSERT INTO transactions (id, project_id, command, created_at) VALUES (?1, ?2, ?3, ?4)",
      params![id, project_id, command, now],
    )?;
    Ok(id)
  }

  /// Record a file mutation event within a transaction.
  pub fn record_event(
    &self,
    transaction_id: &str,
    file_path: &str,
    event_type: EventType,
    before_content: Option<&[u8]>,
  ) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    self.conn.execute(
      "INSERT INTO events (transaction_id, file_path, event_type, before_content, \
        created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![transaction_id, file_path, event_type.as_str(), before_content, now],
    )?;
    Ok(())
  }

  /// Return the most recent non-undone transaction for a project, with its events.
  ///
  /// Returns `None` if there is nothing to undo.
  pub fn latest_undoable(&self, project_id: &str) -> Result<Option<Transaction>> {
    let mut stmt = self.conn.prepare(
      "SELECT id, command, created_at FROM transactions \
        WHERE project_id = ?1 AND undone_at IS NULL ORDER BY created_at DESC \
        LIMIT 1",
    )?;

    let tx = stmt
      .query_row(params![project_id], |row| {
        Ok(TransactionRow {
          id: row.get(0)?,
          command: row.get(1)?,
          created_at: row.get(2)?,
        })
      })
      .optional()?;

    let Some(tx) = tx else {
      return Ok(None);
    };

    let events = self.load_events(&tx.id)?;

    Ok(Some(Transaction {
      id: tx.id,
      command: tx.command,
      created_at: parse_datetime(&tx.created_at),
      events,
    }))
  }

  /// Mark a transaction as undone by setting its `undone_at` timestamp.
  pub fn mark_undone(&self, transaction_id: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    self.conn.execute(
      "UPDATE transactions SET undone_at = ?1 WHERE id = ?2",
      params![now, transaction_id],
    )?;
    Ok(())
  }

  /// Load all events for a given transaction, ordered by ID.
  fn load_events(&self, transaction_id: &str) -> Result<Vec<Event>> {
    let mut stmt = self.conn.prepare(
      "SELECT file_path, event_type, before_content FROM events \
        WHERE transaction_id = ?1 ORDER BY id ASC",
    )?;

    let events = stmt
      .query_map(params![transaction_id], |row| {
        let file_path: String = row.get(0)?;
        let event_type: String = row.get(1)?;
        let before_content: Option<Vec<u8>> = row.get(2)?;
        Ok(Event {
          file_path,
          event_type: EventType::from_str(&event_type),
          before_content,
        })
      })?
      .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(events)
  }
}

/// Intermediate row type for SQLite query results.
struct TransactionRow {
  id: String,
  command: String,
  created_at: String,
}

/// Use `optional()` extension from rusqlite.
trait OptionalExt<T> {
  fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
  fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error> {
    match self {
      Ok(val) => Ok(Some(val)),
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e),
    }
  }
}

fn create_schema(conn: &Connection) -> std::result::Result<(), rusqlite::Error> {
  conn.execute_batch(
    "CREATE TABLE IF NOT EXISTS transactions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  command TEXT NOT NULL,
  created_at TEXT NOT NULL,
  undone_at TEXT
);
CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  transaction_id TEXT NOT NULL REFERENCES transactions(id),
  file_path TEXT NOT NULL,
  event_type TEXT NOT NULL,
  before_content BLOB,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_transactions_project_undone
  ON transactions(project_id, undone_at, created_at);
CREATE INDEX IF NOT EXISTS idx_events_transaction
  ON events(transaction_id);",
  )
}

/// Generate a random 16-character hex string for use as a transaction ID.
fn generate_id() -> String {
  use rand::RngExt;
  let bytes: [u8; 8] = rand::rng().random();
  bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parse an RFC 3339 datetime string, panicking on invalid input (should never
/// happen for data we wrote ourselves).
fn parse_datetime(s: &str) -> DateTime<Utc> {
  DateTime::parse_from_rfc3339(s)
    .expect("invalid datetime in event store")
    .with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_store() -> EventStore {
    EventStore::open_in_memory().unwrap()
  }

  #[test]
  fn it_creates_and_retrieves_a_transaction() {
    let store = make_store();
    let tx_id = store.begin_transaction("proj1", "task create foo").unwrap();

    store
      .record_event(&tx_id, "tasks/abc.toml", EventType::Created, None)
      .unwrap();

    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.id, tx_id);
    assert_eq!(tx.command, "task create foo");
    assert_eq!(tx.events.len(), 1);
    assert_eq!(tx.events[0].file_path, "tasks/abc.toml");
    assert_eq!(tx.events[0].event_type, EventType::Created);
    assert!(tx.events[0].before_content.is_none());
  }

  #[test]
  fn it_stores_before_content_for_modified_events() {
    let store = make_store();
    let tx_id = store.begin_transaction("proj1", "task update foo").unwrap();
    let content = b"title = \"old title\"\n";

    store
      .record_event(&tx_id, "tasks/abc.toml", EventType::Modified, Some(content))
      .unwrap();

    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.events[0].before_content.as_deref(), Some(content.as_slice()));
  }

  #[test]
  fn it_returns_none_when_no_undoable_transactions() {
    let store = make_store();
    assert!(store.latest_undoable("proj1").unwrap().is_none());
  }

  #[test]
  fn it_skips_undone_transactions() {
    let store = make_store();

    let tx1 = store.begin_transaction("proj1", "first command").unwrap();
    store
      .record_event(&tx1, "tasks/a.toml", EventType::Created, None)
      .unwrap();

    let tx2 = store.begin_transaction("proj1", "second command").unwrap();
    store
      .record_event(&tx2, "tasks/b.toml", EventType::Created, None)
      .unwrap();

    // Mark the latest as undone
    store.mark_undone(&tx2).unwrap();

    // Should return the first transaction now
    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.id, tx1);
    assert_eq!(tx.command, "first command");
  }

  #[test]
  fn it_scopes_transactions_by_project() {
    let store = make_store();

    store.begin_transaction("proj1", "command in proj1").unwrap();
    store.begin_transaction("proj2", "command in proj2").unwrap();

    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.command, "command in proj1");

    let tx = store.latest_undoable("proj2").unwrap().unwrap();
    assert_eq!(tx.command, "command in proj2");
  }

  #[test]
  fn it_records_multiple_events_per_transaction() {
    let store = make_store();
    let tx_id = store.begin_transaction("proj1", "advance phase").unwrap();

    store
      .record_event(&tx_id, "tasks/a.toml", EventType::Modified, Some(b"old a"))
      .unwrap();
    store
      .record_event(&tx_id, "tasks/b.toml", EventType::Modified, Some(b"old b"))
      .unwrap();
    store
      .record_event(&tx_id, "tasks/c.toml", EventType::Modified, Some(b"old c"))
      .unwrap();

    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.events.len(), 3);
    assert_eq!(tx.events[0].file_path, "tasks/a.toml");
    assert_eq!(tx.events[2].file_path, "tasks/c.toml");
  }

  #[test]
  fn it_handles_deleted_event_type() {
    let store = make_store();
    let tx_id = store.begin_transaction("proj1", "artifact archive foo").unwrap();

    store
      .record_event(&tx_id, "artifacts/x.md", EventType::Deleted, Some(b"# Old content"))
      .unwrap();

    let tx = store.latest_undoable("proj1").unwrap().unwrap();
    assert_eq!(tx.events[0].event_type, EventType::Deleted);
    assert_eq!(
      tx.events[0].before_content.as_deref(),
      Some(b"# Old content".as_slice())
    );
  }
}
