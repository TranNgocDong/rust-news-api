use axum::{Router, routing::post};

use crate::{
    controllers::auth_controller::{login, register},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}