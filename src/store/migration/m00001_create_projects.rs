//! Initial `projects` table recording each known project root.

use super::Migration;

/// Creates the `projects` table for tracking project root directories.
pub const MIGRATION: Migration = Migration {
  name: "create_projects",
  sql: "
    CREATE TABLE projects (
      id         TEXT PRIMARY KEY NOT NULL,
      root       TEXT NOT NULL UNIQUE,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
  ",
  version: 1,
};
