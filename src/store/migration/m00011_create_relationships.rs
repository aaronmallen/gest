use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_relationships",
  sql: "\
    CREATE TABLE relationships (\
      id          TEXT PRIMARY KEY,\
      rel_type    TEXT NOT NULL,\
      source_id   TEXT NOT NULL,\
      source_type TEXT NOT NULL,\
      target_id   TEXT NOT NULL,\
      target_type TEXT NOT NULL\
    );\
    CREATE INDEX idx_relationships_source ON relationships (source_type, source_id);\
    CREATE INDEX idx_relationships_target ON relationships (target_type, target_id);\
    CREATE UNIQUE INDEX idx_relationships_unique \
      ON relationships (rel_type, source_type, source_id, target_type, target_id);\
  ",
  version: 11,
};
