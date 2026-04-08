//! `sync_digests` table tracking content hashes of synced project files.

use super::Migration;

/// Creates the `sync_digests` table keyed by project and relative path.
pub const MIGRATION: Migration = Migration {
  name: "create_sync_digests",
  sql: "
    CREATE TABLE sync_digests (
      project_id    TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
      relative_path TEXT NOT NULL,
      digest        TEXT NOT NULL,
      created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      synced_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      PRIMARY KEY (project_id, relative_path)
    );
  ",
  version: 14,
};
