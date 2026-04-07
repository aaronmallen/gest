use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_events",
  sql: "\
    CREATE TABLE events (\
      id          TEXT PRIMARY KEY,\
      entity_id   TEXT NOT NULL,\
      entity_type TEXT NOT NULL,\
      author_id   TEXT REFERENCES authors(id),\
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),\
      data        TEXT NOT NULL DEFAULT '{}',\
      description TEXT,\
      event_type  TEXT NOT NULL\
    );\
    CREATE INDEX idx_events_entity ON events (entity_type, entity_id);\
  ",
  version: 10,
};
