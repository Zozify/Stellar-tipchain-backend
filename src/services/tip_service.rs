use crate::db::connection::AppState;
use crate::models::tip::{CreateTipRequest, Tip};
use crate::services::stellar_service::StellarVerifyError;
use crate::validation;

#[derive(Debug)]
pub enum TipError {
    InvalidInput(String),
    CreatorNotFound,
    TransactionNotFound,
    TransactionUnsuccessful,
    DuplicateTransaction,
    StellarUnreachable(String),
    DatabaseError(sqlx::Error),
}

pub async fn create_tip(state: &AppState, req: CreateTipRequest) -> Result<Tip, TipError> {
    // Step 0: validate input shape
    validation::validate_amount(&req.amount).map_err(TipError::InvalidInput)?;
    validation::validate_transaction_hash(&req.transaction_hash).map_err(TipError::InvalidInput)?;

    // Step 1: verify creator exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM creators WHERE username = $1)",
    )
    .bind(&req.username)
    .fetch_one(&state.db)
    .await
    .map_err(TipError::DatabaseError)?;

    if !exists {
        return Err(TipError::CreatorNotFound);
    }

    // Step 2: verify transaction on Stellar
    state
        .stellar
        .verify_transaction(&req.transaction_hash)
        .await
        .map_err(|e| match e {
            StellarVerifyError::NotFound => TipError::TransactionNotFound,
            StellarVerifyError::Unsuccessful => TipError::TransactionUnsuccessful,
            StellarVerifyError::NetworkError(msg) => TipError::StellarUnreachable(msg),
        })?;

    // Step 3: insert tip
    sqlx::query_as::<_, Tip>(
        "INSERT INTO tips (creator_username, amount, transaction_hash)
         VALUES ($1, $2, $3)
         RETURNING *",
    )
    .bind(&req.username)
    .bind(&req.amount)
    .bind(&req.transaction_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23505") {
                return TipError::DuplicateTransaction;
            }
        }
        TipError::DatabaseError(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::stellar_service::StellarService;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn insert_creator(pool: &sqlx::PgPool, username: &str) {
        sqlx::query("INSERT INTO creators (username, wallet_address) VALUES ($1, $2)")
            .bind(username)
            .bind(format!("G{}", "A".repeat(55)))
            .execute(pool)
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn rejects_invalid_amount(pool: sqlx::PgPool) {
        let state = AppState {
            db: pool,
            stellar: StellarService::with_base_url("http://127.0.0.1:1"),
        };
        let req = CreateTipRequest {
            username: "alice".into(),
            amount: "-5".into(),
            transaction_hash: "abc123".into(),
        };

        let result = create_tip(&state, req).await;
        assert!(matches!(result, Err(TipError::InvalidInput(_))));
    }

    #[sqlx::test]
    async fn errors_when_creator_missing(pool: sqlx::PgPool) {
        let state = AppState {
            db: pool,
            stellar: StellarService::with_base_url("http://127.0.0.1:1"),
        };
        let req = CreateTipRequest {
            username: "nobody".into(),
            amount: "5".into(),
            transaction_hash: "abc123".into(),
        };

        let result = create_tip(&state, req).await;
        assert!(matches!(result, Err(TipError::CreatorNotFound)));
    }

    #[sqlx::test]
    async fn errors_when_transaction_not_found_on_chain(pool: sqlx::PgPool) {
        insert_creator(&pool, "alice").await;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/transactions/deadbeef"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let state = AppState {
            db: pool,
            stellar: StellarService::with_base_url(&server.uri()),
        };
        let req = CreateTipRequest {
            username: "alice".into(),
            amount: "5".into(),
            transaction_hash: "deadbeef".into(),
        };

        let result = create_tip(&state, req).await;
        assert!(matches!(result, Err(TipError::TransactionNotFound)));
    }

    #[sqlx::test]
    async fn records_tip_on_successful_verification(pool: sqlx::PgPool) {
        insert_creator(&pool, "alice").await;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/transactions/abc123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"successful": true})),
            )
            .mount(&server)
            .await;

        let state = AppState {
            db: pool,
            stellar: StellarService::with_base_url(&server.uri()),
        };
        let req = CreateTipRequest {
            username: "alice".into(),
            amount: "10.5".into(),
            transaction_hash: "abc123".into(),
        };

        let tip = create_tip(&state, req).await.unwrap();
        assert_eq!(tip.creator_username, "alice");
        assert_eq!(tip.amount, "10.5");
    }
}
