use sqlx::PgPool;
use crate::services::stellar_service::StellarService;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub stellar: StellarService,
}
