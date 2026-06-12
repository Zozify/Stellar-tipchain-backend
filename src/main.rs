mod db;
mod models;
mod services;
// TODO: mod controllers;
// TODO: mod routes;

use db::connection::AppState;
use services::stellar_service::StellarService;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let network = std::env::var("STELLAR_NETWORK").unwrap_or_else(|_| "testnet".to_string());

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let _state = AppState {
        db,
        stellar: StellarService::new(&network),
    };

    // TODO: build router, add CORS, start server
    println!("DB connected and migrations applied. Server not yet wired up.");
}
