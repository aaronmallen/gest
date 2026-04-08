//! Server-sent events endpoint that pushes reload pings to connected browser tabs.

use std::convert::Infallible;

use axum::{
  extract::State,
  response::sse::{Event, KeepAlive, Sse},
};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use super::AppState;

/// SSE endpoint that streams named `ping` events to connected clients.
pub async fn events(State(state): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
  let rx = state.reload_tx().subscribe();
  let stream = BroadcastStream::new(rx).map(|_| Ok(Event::default().event("ping").data("reload")));
  Sse::new(stream).keep_alive(KeepAlive::default())
}
