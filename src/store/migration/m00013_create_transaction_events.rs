//! `transaction_events` table capturing individual row changes per transaction.

use crate::store::migration::Migration;

/// Creates the `transaction_events` table with an index on `transaction_id`.
pub const MIGRATION: Migration = Migration {
  name: "create_transaction_events",
  sql: "
    CREATE TABLE transaction_events (
        id             TEXT PRIMARY KEY,
        transaction_id TEXT NOT NULL REFERENCES transactions(id),
        before_data    TEXT,
        event_type     TEXT NOT NULL,
        row_id         TEXT NOT NULL,
        table_name     TEXT NOT NULL,
        semantic_type  TEXT,
        old_value      TEXT,
        new_value      TEXT,
        created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
      );
      CREATE INDEX idx_transaction_events_transaction_id ON transaction_events (transaction_id);
  ",
  version: 13,
};
