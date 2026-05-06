use sqlx::PgPool;

// TODO: add StellarService once services/ is implemented
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
