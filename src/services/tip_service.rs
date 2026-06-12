use crate::db::connection::AppState;
use crate::models::tip::{CreateTipRequest, Tip};
use crate::services::stellar_service::StellarVerifyError;

#[derive(Debug)]
pub enum TipError {
    CreatorNotFound,
    TransactionNotFound,
    TransactionUnsuccessful,
    DuplicateTransaction,
    StellarUnreachable(String),
    DatabaseError(sqlx::Error),
}

pub async fn create_tip(state: &AppState, req: CreateTipRequest) -> Result<Tip, TipError> {
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
