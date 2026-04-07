use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "create_tags",
  sql: "\
    CREATE TABLE tags (\
      id    TEXT PRIMARY KEY,\
      label TEXT NOT NULL UNIQUE\
    );\
  ",
  version: 7,
};
