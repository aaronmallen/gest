//! `tasks` table representing work items tracked within a project.

use super::Migration;

/// Creates the `tasks` table along with indexes on project and status.
pub const MIGRATION: Migration = Migration {
  name: "create_tasks",
  sql: "
    CREATE TABLE tasks (
      id          TEXT PRIMARY KEY,
      project_id  TEXT NOT NULL REFERENCES projects(id),
      title       TEXT NOT NULL,
      priority    INTEGER,
      status      TEXT NOT NULL DEFAULT 'open',
      description TEXT NOT NULL DEFAULT '',
      assigned_to TEXT REFERENCES authors(id),
      metadata    TEXT NOT NULL DEFAULT '{}',
      resolved_at TEXT,
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    );
    CREATE INDEX idx_tasks_project_id ON tasks (project_id);
    CREATE INDEX idx_tasks_status ON tasks (project_id, status);
  ",
  version: 4,
};
