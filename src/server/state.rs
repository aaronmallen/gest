//! Server-side application state shared across all request handlers.

use crate::config::Settings;

/// Shared state available to every handler via Axum's state extractor.
#[derive(Clone, Debug)]
pub struct ServerState {
  pub settings: Settings,
}
