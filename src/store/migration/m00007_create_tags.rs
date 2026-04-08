//! `tags` table holding the global set of labels applied to entities.

use super::Migration;

/// Creates the `tags` table keyed by a unique `label`.
pub const MIGRATION: Migration = Migration {
  name: "create_tags",
  sql: "
    CREATE TABLE tags (
      id         TEXT PRIMARY KEY,
      label      TEXT NOT NULL UNIQUE,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
  ",
  version: 7,
};
