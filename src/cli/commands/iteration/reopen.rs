use clap::Args;

use crate::{
  AppContext,
  actions::{Iteration, transition::transition_status},
  cli::Error,
  store::model::primitives::IterationStatus,
  ui::json,
};

/// Reopen a completed or cancelled iteration.
#[derive(Args, Debug)]
pub struct Command {
  /// The iteration ID or prefix.
  id: String,
  #[command(flatten)]
  output: json::Flags,
}

impl Command {
  /// Transition the iteration back to `active` within a recorded transaction.
  pub async fn call(&self, context: &AppContext) -> Result<(), Error> {
    transition_status::<Iteration>(
      context,
      &self.id,
      IterationStatus::Active,
      "iteration reopen",
      "status-change",
      "reopened iteration",
      &self.output,
    )
    .await
  }
}
