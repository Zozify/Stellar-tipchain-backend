use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::tip_controller::{create_tip, list_tips};
use crate::db::connection::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tips", post(create_tip))
        .route("/creators/:username/tips", get(list_tips))
}
