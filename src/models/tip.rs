use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Tip {
    pub id: Uuid,
    pub creator_username: String,
    pub amount: String,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTipRequest {
    pub username: String,
    pub amount: String,
    pub transaction_hash: String,
}
