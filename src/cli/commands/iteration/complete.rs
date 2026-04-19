use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, transition::transition_status},
  cli::Error,
  store::model::primitives::IterationStatus,
  ui::json,
};

/// Complete an iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Transition the iteration to `completed` within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    transition_status::<Iteration>(
      context,
      &self.id,
      IterationStatus::Completed,
      "iteration complete",
      "completed",
      "completed iteration",
      &self.output,
    )
    .await
  }
}
