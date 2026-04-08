//! `iterations` table grouping tasks into time-boxed units of work.

use super::Migration;

/// Creates the `iterations` table with indexes on project and status.
pub const MIGRATION: Migration = Migration {
  name: "create_iterations",
  sql: "
    CREATE TABLE iterations (
      id           TEXT PRIMARY KEY,
      project_id   TEXT NOT NULL REFERENCES projects(id),
      title        TEXT NOT NULL,
      status       TEXT NOT NULL DEFAULT 'active',
      description  TEXT NOT NULL DEFAULT '',
      metadata     TEXT NOT NULL DEFAULT '{}',
      completed_at TEXT,
      created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
    CREATE INDEX idx_iterations_project_id ON iterations (project_id);
    CREATE INDEX idx_iterations_status ON iterations (project_id, status);
  ",
  version: 6,
};
