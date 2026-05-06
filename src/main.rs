mod db;
mod models;
// TODO: mod controllers;
// TODO: mod routes;
// TODO: mod services;

use db::connection::AppState;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let _state = AppState { db };

    // TODO: build router, add CORS, start server
    println!("DB connected and migrations applied. Server not yet wired up.");
}
