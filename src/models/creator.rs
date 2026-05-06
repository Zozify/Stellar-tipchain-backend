use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Creator {
    pub id: Uuid,
    pub username: String,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreatorRequest {
    pub username: String,
    pub wallet_address: String,
}
