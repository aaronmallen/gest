//! `iteration_tasks` join table assigning tasks to iteration phases.

use super::Migration;

/// Creates the `iteration_tasks` join table with an index on `task_id`.
pub const MIGRATION: Migration = Migration {
  name: "create_iteration_tasks",
  sql: "
    CREATE TABLE iteration_tasks (
      iteration_id TEXT NOT NULL REFERENCES iterations(id),
      task_id      TEXT NOT NULL REFERENCES tasks(id),
      phase        INTEGER NOT NULL DEFAULT 1,
      created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      PRIMARY KEY (iteration_id, task_id)
    );
    CREATE INDEX idx_iteration_tasks_task_id ON iteration_tasks (task_id);
  ",
  version: 11,
};
