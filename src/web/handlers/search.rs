//! Search handlers (HTML page + JSON API).

use askama::Template;
use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
  store::{
    model::{artifact, iteration, task},
    repo, search_query,
  },
  web::{AppState, handlers::log_err},
};

#[derive(Deserialize)]
pub struct ApiSearchParams {
  q: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
  q: Option<String>,
}

/// JSON result returned from `/api/search`.
#[derive(Serialize)]
struct ApiSearchResult {
  id: String,
  #[serde(rename = "type")]
  kind: String,
  short_id: String,
  title: String,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
  artifacts: Vec<artifact::Model>,
  iterations: Vec<iteration::Model>,
  query: String,
  tasks: Vec<task::Model>,
}

/// GET /api/search?q=... — JSON search results for the relationship picker.
pub async fn api_search(State(state): State<AppState>, Query(params): Query<ApiSearchParams>) -> Response {
  let query = params.q.unwrap_or_default();
  if query.is_empty() {
    return axum::Json(Vec::<ApiSearchResult>::new()).into_response();
  }

  let conn = match state.store().connect().await {
    Ok(c) => c,
    Err(e) => {
      log::error!("api search connect failed: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(Vec::<ApiSearchResult>::new()),
      )
        .into_response();
    }
  };

  let parsed = search_query::parse(&query);
  let results = match repo::search::query(&conn, state.project_id(), &parsed, true).await {
    Ok(r) => r,
    Err(e) => {
      log::error!("api search failed: {e}");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(Vec::<ApiSearchResult>::new()),
      )
        .into_response();
    }
  };

  let mut items: Vec<ApiSearchResult> = Vec::new();
  for task in results.tasks {
    items.push(ApiSearchResult {
      id: task.id().to_string(),
      kind: "task".to_string(),
      short_id: task.id().short(),
      title: task.title().to_owned(),
    });
  }
  for artifact in results.artifacts {
    items.push(ApiSearchResult {
      id: artifact.id().to_string(),
      kind: "artifact".to_string(),
      short_id: artifact.id().short(),
      title: artifact.title().to_owned(),
    });
  }
  for iteration in results.iterations {
    items.push(ApiSearchResult {
      id: iteration.id().to_string(),
      kind: "iteration".to_string(),
      short_id: iteration.id().short(),
      title: iteration.title().to_owned(),
    });
  }

  axum::Json(items).into_response()
}

/// Search page.
pub async fn search(State(state): State<AppState>, Query(params): Query<SearchQuery>) -> Result<Html<String>, String> {
  let conn = state.store().connect().await.map_err(log_err("search"))?;
  let query = params.q.unwrap_or_default();

  let (tasks, artifacts, iterations) = if query.is_empty() {
    (Vec::new(), Vec::new(), Vec::new())
  } else {
    let parsed = search_query::parse(&query);
    let results = repo::search::query(&conn, state.project_id(), &parsed, true)
      .await
      .map_err(log_err("search"))?;
    (results.tasks, results.artifacts, results.iterations)
  };

  let tmpl = SearchTemplate {
    artifacts,
    iterations,
    query,
    tasks,
  };
  Ok(Html(tmpl.render().map_err(log_err("search"))?))
}
