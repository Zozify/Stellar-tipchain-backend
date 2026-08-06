use axum::{
    routing::{get, post},
    Router,
};

use crate::controllers::creator_controller::{create_creator, get_creator};
use crate::db::connection::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/creators", post(create_creator))
        .route("/creators/:username", get(get_creator))
}
