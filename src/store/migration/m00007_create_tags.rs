use super::Migration;

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
