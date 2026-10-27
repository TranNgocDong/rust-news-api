pub mod auth_routes;
pub mod file_routes;
pub mod news_routes;
pub mod user_routes;

use axum::Router;

use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/api/auth", auth_routes::router())
        .nest("/api/users", user_routes::router(state.clone()))
        .nest("/api/news", news_routes::router(state.clone()))
        .nest("/api/files", file_routes::router(state))
}
