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
use crate::validation;

pub async fn create_creator(
    State(state): State<AppState>,
    Json(body): Json<CreateCreatorRequest>,
) -> impl IntoResponse {
    if let Err(msg) = validation::validate_username(&body.username) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response();
    }
    if let Err(msg) = validation::validate_stellar_address(&body.wallet_address) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response();
    }

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
        Err(err) => {
            tracing::error!(error = %err, "database error while creating creator");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal server error"})),
            )
                .into_response()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stellar_service::StellarService;

    fn test_state(pool: sqlx::PgPool) -> AppState {
        AppState {
            db: pool,
            stellar: StellarService::with_base_url("http://127.0.0.1:1"),
        }
    }

    fn valid_wallet() -> String {
        format!("G{}", "A".repeat(55))
    }

    #[sqlx::test]
    async fn creates_and_fetches_creator(pool: sqlx::PgPool) {
        let state = test_state(pool);

        let create_resp = create_creator(
            State(state.clone()),
            Json(CreateCreatorRequest {
                username: "alice".into(),
                wallet_address: valid_wallet(),
            }),
        )
        .await
        .into_response();
        assert_eq!(create_resp.status(), StatusCode::CREATED);

        let get_resp = get_creator(State(state), Path("alice".into()))
            .await
            .into_response();
        assert_eq!(get_resp.status(), StatusCode::OK);
    }

    #[sqlx::test]
    async fn rejects_invalid_username(pool: sqlx::PgPool) {
        let state = test_state(pool);

        let resp = create_creator(
            State(state),
            Json(CreateCreatorRequest {
                username: "a".into(),
                wallet_address: valid_wallet(),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test]
    async fn rejects_duplicate_username(pool: sqlx::PgPool) {
        let state = test_state(pool);

        create_creator(
            State(state.clone()),
            Json(CreateCreatorRequest {
                username: "alice".into(),
                wallet_address: valid_wallet(),
            }),
        )
        .await;

        let resp = create_creator(
            State(state),
            Json(CreateCreatorRequest {
                username: "alice".into(),
                wallet_address: valid_wallet(),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[sqlx::test]
    async fn get_unknown_creator_returns_404(pool: sqlx::PgPool) {
        let state = test_state(pool);

        let resp = get_creator(State(state), Path("nobody".into()))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
