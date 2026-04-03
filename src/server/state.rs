//! Server-side application state shared across all request handlers.

use std::sync::Arc;

use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

use crate::config::Settings;

/// Shared state available to every handler via Axum's state extractor.
#[derive(Clone, Debug)]
pub struct ServerState {
  pub settings: Settings,
  ping_tx: Arc<Sender<()>>,
}

impl ServerState {
  /// Creates a new `ServerState` with the given settings and an internal broadcast channel.
  pub fn new(settings: Settings) -> Self {
    let (ping_tx, _) = tokio::sync::broadcast::channel(16);
    Self {
      settings,
      ping_tx: Arc::new(ping_tx),
    }
  }

  /// Returns a new receiver that will observe future ping notifications.
  pub fn subscribe_pings(&self) -> Receiver<()> {
    self.ping_tx.subscribe()
  }

  /// Sends a ping notification to all active subscribers.
  pub fn send_ping(&self) {
    // Ignore the error — it just means no receivers are listening.
    let _ = self.ping_tx.send(());
  }
}
