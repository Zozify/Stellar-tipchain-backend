use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::db::connection::AppState;
use crate::models::tip::{CreateTipRequest, Tip};
use crate::services::tip_service::{self, TipError};

pub async fn create_tip(
    State(state): State<AppState>,
    Json(body): Json<CreateTipRequest>,
) -> impl IntoResponse {
    match tip_service::create_tip(&state, body).await {
        Ok(tip) => (StatusCode::CREATED, Json(json!(tip))).into_response(),
        Err(TipError::InvalidInput(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
        Err(TipError::CreatorNotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "creator not found"})),
        )
            .into_response(),
        Err(TipError::TransactionNotFound) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "transaction not found on network"})),
        )
            .into_response(),
        Err(TipError::TransactionUnsuccessful) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "transaction was not successful"})),
        )
            .into_response(),
        Err(TipError::DuplicateTransaction) => (
            StatusCode::CONFLICT,
            Json(json!({"error": "transaction already recorded"})),
        )
            .into_response(),
        Err(TipError::StellarUnreachable(reason)) => {
            tracing::error!(reason = %reason, "stellar network unreachable while verifying tip");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "stellar network unreachable"})),
            )
                .into_response()
        }
        Err(TipError::DatabaseError(_)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal server error"})),
        )
            .into_response(),
    }
}

pub async fn list_tips(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let creator_exists: bool =
        match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM creators WHERE username = $1)")
            .bind(&username)
            .fetch_one(&state.db)
            .await
        {
            Ok(v) => v,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "internal server error"})),
                )
                    .into_response()
            }
        };

    if !creator_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "creator not found"})),
        )
            .into_response();
    }

    match sqlx::query_as::<_, Tip>(
        "SELECT * FROM tips WHERE creator_username = $1 ORDER BY created_at DESC",
    )
    .bind(&username)
    .fetch_all(&state.db)
    .await
    {
        Ok(tips) => (StatusCode::OK, Json(json!(tips))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "internal server error"})),
        )
            .into_response(),
    }
}
