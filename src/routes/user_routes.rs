use axum::{
    Router, middleware,
    routing::{get, put},
};

use crate::{
    controllers::user_controller::{get_me, list_users, update_permissions},
    middleware::auth_middleware::auth_middleware,
    state::AppState,
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/", get(list_users))
        .route("/{id}/permissions", put(update_permissions))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}
