use super::Migration;

pub const MIGRATION: Migration = Migration {
  name: "extend_transaction_events",
  sql: "\
    ALTER TABLE transaction_events ADD COLUMN semantic_type TEXT;\
    ALTER TABLE transaction_events ADD COLUMN old_value TEXT;\
    ALTER TABLE transaction_events ADD COLUMN new_value TEXT;\
    DROP INDEX IF EXISTS idx_events_entity;\
    DROP TABLE IF EXISTS events;\
  ",
  version: 15,
};
