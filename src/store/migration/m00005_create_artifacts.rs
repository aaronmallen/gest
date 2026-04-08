//! `artifacts` table storing long-form documents attached to a project.

use super::Migration;

/// Creates the `artifacts` table with an index on `project_id`.
pub const MIGRATION: Migration = Migration {
  name: "create_artifacts",
  sql: "
    CREATE TABLE artifacts (
      id          TEXT PRIMARY KEY,
      project_id  TEXT NOT NULL REFERENCES projects(id),
      title       TEXT NOT NULL,
      body        TEXT NOT NULL DEFAULT '',
      metadata    TEXT NOT NULL DEFAULT '{}',
      archived_at TEXT,
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
    CREATE INDEX idx_artifacts_project_id ON artifacts (project_id);
  ",
  version: 5,
};
