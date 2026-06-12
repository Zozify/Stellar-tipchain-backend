mod controllers;
mod db;
mod models;
mod services;

use axum::{
    routing::{get, post},
    Router,
};
use db::connection::AppState;
use services::stellar_service::StellarService;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "stellar_tipchain_backend=debug,tower_http=debug".into()),
        )
        .init();

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

    let state = AppState {
        db,
        stellar: StellarService::new(&network),
    };

    let app = Router::new()
        .route(
            "/creators",
            post(controllers::creator_controller::create_creator),
        )
        .route(
            "/creators/:username",
            get(controllers::creator_controller::get_creator),
        )
        .route("/tips", post(controllers::tip_controller::create_tip))
        .route(
            "/creators/:username/tips",
            get(controllers::tip_controller::list_tips),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".into());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
