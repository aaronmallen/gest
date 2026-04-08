//! `entity_tags` join table associating tags with arbitrary entity types.

use super::Migration;

/// Creates the `entity_tags` join table and an index on `tag_id`.
pub const MIGRATION: Migration = Migration {
  name: "create_entity_tags",
  sql: "
    CREATE TABLE entity_tags (
      entity_id   TEXT NOT NULL,
      entity_type TEXT NOT NULL,
      tag_id      TEXT NOT NULL REFERENCES tags(id),
      created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      PRIMARY KEY (entity_type, entity_id, tag_id)
    );
    CREATE INDEX idx_entity_tags_tag_id ON entity_tags (tag_id);
  ",
  version: 8,
};
