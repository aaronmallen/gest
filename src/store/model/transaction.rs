use chrono::{DateTime, Utc};
use libsql::Row;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Error, primitives::Id};

/// A recorded command execution for undo support.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  command: String,
  created_at: DateTime<Utc>,
  id: Id,
  project_id: Id,
  undone_at: Option<DateTime<Utc>>,
}

impl Model {
  /// The command string that was executed.
  pub fn command(&self) -> &str {
    &self.command
  }

  /// When this transaction was recorded.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// The unique identifier for this transaction.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The project this transaction belongs to.
  pub fn project_id(&self) -> &Id {
    &self.project_id
  }

  /// When this transaction was undone, if at all.
  pub fn undone_at(&self) -> Option<&DateTime<Utc>> {
    self.undone_at.as_ref()
  }
}

/// Expects columns in order: `id`, `project_id`, `command`, `created_at`, `undone_at`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let project_id: String = row.get(1)?;
    let command: String = row.get(2)?;
    let created_at: String = row.get(3)?;
    let undone_at: Option<String> = row.get(4)?;

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
      command,
      created_at,
      id,
      project_id,
      undone_at,
    })
  }
}

/// A single change recorded within a transaction for undo replay.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event {
  before_data: Option<Value>,
  created_at: DateTime<Utc>,
  event_type: String,
  id: Id,
  new_value: Option<String>,
  old_value: Option<String>,
  row_id: String,
  semantic_type: Option<String>,
  table_name: String,
  transaction_id: Id,
}

impl Event {
  /// The state of the row before the change, for undo replay.
  pub fn before_data(&self) -> Option<&Value> {
    self.before_data.as_ref()
  }

  /// When this event was recorded.
  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  /// The type of change (e.g. "created", "modified", "deleted").
  pub fn event_type(&self) -> &str {
    &self.event_type
  }

  /// The unique identifier for this event.
  pub fn id(&self) -> &Id {
    &self.id
  }

  /// The new value for the semantic change, if applicable.
  pub fn new_value(&self) -> Option<&str> {
    self.new_value.as_deref()
  }

  /// The previous value for the semantic change, if applicable.
  pub fn old_value(&self) -> Option<&str> {
    self.old_value.as_deref()
  }

  /// The ID of the row that was changed.
  pub fn row_id(&self) -> &str {
    &self.row_id
  }

  /// The semantic kind of change (e.g. `"status-change"`, `"created"`),
  /// if this event corresponds to a user-facing activity entry.
  pub fn semantic_type(&self) -> Option<&str> {
    self.semantic_type.as_deref()
  }

  /// The database table that was modified.
  pub fn table_name(&self) -> &str {
    &self.table_name
  }

  /// The transaction this event belongs to.
  pub fn transaction_id(&self) -> &Id {
    &self.transaction_id
  }
}

/// Expects columns in order: `id`, `transaction_id`, `before_data`, `created_at`,
/// `event_type`, `row_id`, `table_name`, `semantic_type`, `old_value`, `new_value`.
impl TryFrom<Row> for Event {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let id: String = row.get(0)?;
    let transaction_id: String = row.get(1)?;
    let before_data: Option<String> = row.get(2)?;
    let created_at: String = row.get(3)?;
    let event_type: String = row.get(4)?;
    let row_id: String = row.get(5)?;
    let table_name: String = row.get(6)?;
    let semantic_type: Option<String> = row.get(7)?;
    let old_value: Option<String> = row.get(8)?;
    let new_value: Option<String> = row.get(9)?;

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
