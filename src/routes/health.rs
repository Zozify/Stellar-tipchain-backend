use axum::{http::StatusCode, routing::get, Json, Router};
use serde_json::json;

use crate::db::connection::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
