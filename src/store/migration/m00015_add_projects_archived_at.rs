//! Add nullable `archived_at` timestamp to the `projects` table.

use super::Migration;

/// Adds the `archived_at` column for soft-archive support on projects.
pub const MIGRATION: Migration = Migration {
  name: "add_projects_archived_at",
  sql: "ALTER TABLE projects ADD COLUMN archived_at TEXT;",
  version: 15,
};
