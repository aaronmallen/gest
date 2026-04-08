//! `transactions` table recording user-facing commands for undo history.

use super::Migration;

/// Creates the `transactions` table with an index on `project_id`.
pub const MIGRATION: Migration = Migration {
  name: "create_transactions",
  sql: "
    CREATE TABLE transactions (
      id         TEXT PRIMARY KEY,
      project_id TEXT NOT NULL REFERENCES projects(id),
      author_id  TEXT REFERENCES authors(id),
      command    TEXT NOT NULL,
      undone_at  TEXT,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
    CREATE INDEX idx_transactions_project_id ON transactions (project_id);
  ",
  version: 12,
};
