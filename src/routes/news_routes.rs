use axum::{Router, routing::get};

use crate::{
    controllers::news_controller::{get_news, list_news},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_news))
        .route("/{id}", get(get_news))
}