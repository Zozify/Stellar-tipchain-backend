use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db::connection::AppState;
use crate::models::creator::{CreateCreatorRequest, Creator};

pub async fn create_creator(
    State(state): State<AppState>,
    Json(body): Json<CreateCreatorRequest>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Creator>(
        "INSERT INTO creators (id, username, wallet_address, created_at)
         VALUES ($1, $2, $3, NOW())
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&body.username)
    .bind(&body.wallet_address)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(creator) => (StatusCode::CREATED, Json(json!(creator))).into_response(),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => (
            StatusCode::CONFLICT,
            Json(json!({"error": "username already taken"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal server error"})),
        )
            .into_response(),
    }
}

pub async fn get_creator(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Creator>("SELECT * FROM creators WHERE username = $1")
        .bind(&username)
        .fetch_one(&state.db)
        .await;

    match result {
        Ok(creator) => (StatusCode::OK, Json(json!(creator))).into_response(),
        Err(sqlx::Error::RowNotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "creator not found"})),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal server error"})),
        )
            .into_response(),
    }
}
