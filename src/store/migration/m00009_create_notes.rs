use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_notes",
  sql: "\
    CREATE TABLE notes (\
      id          TEXT PRIMARY KEY,\
      entity_id   TEXT NOT NULL,\
      entity_type TEXT NOT NULL,\
      author_id   TEXT REFERENCES authors(id),\
      body        TEXT NOT NULL,\
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))\
    );\
    CREATE INDEX idx_notes_entity ON notes (entity_type, entity_id);\
  ",
  version: 9,
};
