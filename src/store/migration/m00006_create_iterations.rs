use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_iterations",
  sql: "\
    CREATE TABLE iterations (\
      id           TEXT PRIMARY KEY,\
      project_id   TEXT NOT NULL REFERENCES projects(id),\
      completed_at TEXT,\
      created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      description  TEXT NOT NULL DEFAULT '',\
      metadata     TEXT NOT NULL DEFAULT '{}',\
      status       TEXT NOT NULL DEFAULT 'active',\
      title        TEXT NOT NULL,\
      updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
    );\
    CREATE INDEX idx_iterations_project_id ON iterations (project_id);\
    CREATE INDEX idx_iterations_status ON iterations (project_id, status);\
  ",
  version: 6,
};
