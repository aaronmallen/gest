use super::Migration;

/// Recreate `sync_digests` so its primary key is the repo-relative path
/// (e.g. `task/abc.yaml`) instead of an absolute filesystem path. The cached
/// digests from the old shared-file layout are dropped — the on-disk format
/// is changing under ADR-0016 anyway, so they would all be stale.
pub const MIGRATION: Migration = Migration {
  name: "recreate_sync_digests_repo_relative",
  sql: "\
    DROP TABLE IF EXISTS sync_digests;\
    CREATE TABLE sync_digests (\
      project_id    TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,\
      relative_path TEXT NOT NULL,\
      digest        TEXT NOT NULL,\
      synced_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      PRIMARY KEY (project_id, relative_path)\
    );\
  ",
  version: 17,
};
