use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_sync_digests",
  sql: "\
    CREATE TABLE sync_digests (\
      file_path  TEXT PRIMARY KEY,\
      project_id TEXT NOT NULL REFERENCES projects(id),\
      digest     TEXT NOT NULL,\
      synced_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
    );\
  ",
  version: 14,
};
