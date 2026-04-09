use chrono::{DateTime, Utc};
use getset::Getters;
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::primitives::Id;
use crate::store::Error;

/// A recorded command execution for undo support.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Model {
  /// Author who ran the command, if known.
  #[get = "pub"]
  author_id: Option<Id>,
  /// Rendered command line as executed.
  #[get = "pub"]
  command: String,
  /// When the transaction was recorded.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Stable identifier assigned at creation.
  #[get = "pub"]
  id: Id,
  /// Project the command ran against.
  #[get = "pub"]
  project_id: Id,
  /// When the transaction was undone, or `None` if still in effect.
  #[get = "pub"]
  undone_at: Option<DateTime<Utc>>,
}

/// Converts a database row into a [`Model`].
///
/// Expects columns in order: `id`, `project_id`, `author_id`, `command`,
/// `undone_at`, `created_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let author_id: Option<String> = row.get(2)?;
    let command: String = row.get(3)?;
    let undone_at: Option<String> = row.get(4)?;
    let created_at: String = row.get(5)?;

    let author_id = author_id
      .map(|s| s.parse::<Id>())
      .transpose()
      .map_err(Error::InvalidValue)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let project_id: Id = project_id.parse().map_err(Error::InvalidValue)?;
    let undone_at = undone_at
      .map(|s| {
        DateTime::parse_from_rfc3339(&s)
          .map(|dt| dt.with_timezone(&Utc))
          .map_err(|e| Error::InvalidValue(e.to_string()))
      })
      .transpose()?;

    Ok(Self {
      author_id,
      command,
      created_at,
      id,
      project_id,
      undone_at,
    })
  }
}

/// A single change recorded within a transaction for undo replay.
#[derive(Clone, Debug, Deserialize, Eq, Getters, PartialEq, Serialize)]
pub struct Event {
  /// Full JSON snapshot of the affected row before the change, when captured.
  #[get = "pub"]
  before_data: Option<Value>,
  /// When the event was recorded.
  #[get = "pub"]
  created_at: DateTime<Utc>,
  /// Kind of change (e.g. `insert`, `update`, `delete`).
  #[get = "pub"]
  event_type: String,
  /// Stable identifier assigned at creation.
  #[get = "pub"]
  id: Id,
  /// Serialized post-change value for single-field updates, when applicable.
  #[get = "pub"]
  new_value: Option<String>,
  /// Serialized pre-change value for single-field updates, when applicable.
  #[get = "pub"]
  old_value: Option<String>,
  /// Primary key of the affected row.
  #[get = "pub"]
  row_id: String,
  /// Optional domain-level column name the change targets.
  #[get = "pub"]
  semantic_type: Option<String>,
  /// Name of the affected table.
  #[get = "pub"]
  table_name: String,
  /// Transaction this event belongs to.
  #[get = "pub"]
  transaction_id: Id,
}

/// Converts a database row into an [`Event`].
///
/// Expects columns in order: `id`, `transaction_id`, `before_data`, `event_type`,
/// `row_id`, `table_name`, `semantic_type`, `old_value`, `new_value`, `created_at`.
impl TryFrom<Row> for Event {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let transaction_id: String = row.get(1)?;
    let before_data: Option<String> = row.get(2)?;
    let event_type: String = row.get(3)?;
    let row_id: String = row.get(4)?;
    let table_name: String = row.get(5)?;
    let semantic_type: Option<String> = row.get(6)?;
    let old_value: Option<String> = row.get(7)?;
    let new_value: Option<String> = row.get(8)?;
    let created_at: String = row.get(9)?;

    let before_data = before_data
      .map(|s| serde_json::from_str(&s).map_err(|e| Error::InvalidValue(e.to_string())))
      .transpose()?;
    let created_at = DateTime::parse_from_rfc3339(&created_at)
      .map(|dt| dt.with_timezone(&Utc))
      .map_err(|e| Error::InvalidValue(e.to_string()))?;
    let id: Id = id.parse().map_err(Error::InvalidValue)?;
    let transaction_id: Id = transaction_id.parse().map_err(Error::InvalidValue)?;

    Ok(Self {
      before_data,
      created_at,
      event_type,
      id,
      new_value,
      old_value,
      row_id,
      semantic_type,
      table_name,
      transaction_id,
    })
  }
}
