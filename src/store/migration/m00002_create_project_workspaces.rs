//! `project_workspaces` table mapping additional checkouts to a project.

use super::Migration;

/// Creates the `project_workspaces` table and a lookup index on `path`.
pub const MIGRATION: Migration = Migration {
  name: "create_project_workspaces",
  sql: "
    CREATE TABLE project_workspaces (
      id         TEXT PRIMARY KEY NOT NULL,
      project_id TEXT NOT NULL REFERENCES projects(id),
      path       TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      UNIQUE(path, project_id)
    );

    CREATE INDEX idx_project_workspaces_path ON project_workspaces(path);
  ",
  version: 2,
};
