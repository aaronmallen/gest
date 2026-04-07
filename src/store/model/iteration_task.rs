use libsql::Row;
use serde::{Deserialize, Serialize};

use super::{Error, primitives::Id};

/// Associates a task with an iteration at a specific phase.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Model {
  iteration_id: Id,
  phase: u32,
  task_id: Id,
}

impl Model {
  /// Create a new iteration-task association.
  pub fn new(iteration_id: Id, task_id: Id, phase: u32) -> Self {
    Self {
      iteration_id,
      phase,
      task_id,
    }
  }

  /// The iteration this task belongs to.
  pub fn iteration_id(&self) -> &Id {
    &self.iteration_id
  }

  /// The phase within the iteration.
  pub fn phase(&self) -> u32 {
    self.phase
  }

  /// The task associated with the iteration.
  pub fn task_id(&self) -> &Id {
    &self.task_id
  }
}

/// Expects columns in order: `iteration_id`, `phase`, `task_id`.
impl TryFrom<Row> for Model {
  type Error = Error;

  fn try_from(row: Row) -> Result<Self, Self::Error> {
    let iteration_id: String = row.get(0)?;
    let phase: i64 = row.get(1)?;
    let task_id: String = row.get(2)?;

    let iteration_id: Id = iteration_id.parse().map_err(Error::InvalidValue)?;
    let task_id: Id = task_id.parse().map_err(Error::InvalidValue)?;

    Ok(Self {
      iteration_id,
      phase: phase as u32,
      task_id,
    })
  }
}
