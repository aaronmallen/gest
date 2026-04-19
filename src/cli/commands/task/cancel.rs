use clap::Args;

use crate::{
  AppContext,
  actions::{Task, transition::transition_status},
  cli::Error,
  store::model::primitives::TaskStatus,
  ui::json,
};

/// Cancel a task.
#[derive(Args, Debug)]
pub struct Command {
  /// The task ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Transition the resolved task to `cancelled` within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    transition_status::<Task>(
      context,
      &self.id,
      TaskStatus::Cancelled,
      "task cancel",
      "cancelled",
      "cancelled task",
      &self.output,
    )
    .await
  }
}
