use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_transactions",
  sql: "\
    CREATE TABLE transactions (\
      id         TEXT PRIMARY KEY,\
      project_id TEXT NOT NULL REFERENCES projects(id),\
      command    TEXT NOT NULL,\
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      undone_at  TEXT\
    );\
    CREATE INDEX idx_transactions_project_id ON transactions (project_id);\
    \
    CREATE TABLE transaction_events (\
      id             TEXT PRIMARY KEY,\
      transaction_id TEXT NOT NULL REFERENCES transactions(id),\
      before_data    TEXT,\
      created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      event_type     TEXT NOT NULL,\
      row_id         TEXT NOT NULL,\
      table_name     TEXT NOT NULL\
    );\
    CREATE INDEX idx_transaction_events_transaction_id ON transaction_events (transaction_id);\
  ",
  version: 13,
};
