//! `authors` table for humans and agents referenced by other records.

use super::Migration;

/// Creates the `authors` table with a unique index on `(name, email)`.
pub const MIGRATION: Migration = Migration {
  name: "create_authors",
  sql: "
    CREATE TABLE authors (
      id          TEXT PRIMARY KEY,
      author_type TEXT NOT NULL DEFAULT 'human',
      name        TEXT NOT NULL,
      email       TEXT,
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
    CREATE UNIQUE INDEX idx_authors_name_email ON authors (name, email);
  ",
  version: 3,
};
