use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_entity_tags",
  sql: "\
    CREATE TABLE entity_tags (\
      entity_id   TEXT NOT NULL,\
      entity_type TEXT NOT NULL,\
      tag_id      TEXT NOT NULL REFERENCES tags(id),\
      PRIMARY KEY (entity_type, entity_id, tag_id)\
    );\
    CREATE INDEX idx_entity_tags_tag_id ON entity_tags (tag_id);\
  ",
  version: 8,
};
