use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, post},
};

use crate::{
    controllers::file_controller::{delete_file, upload_file},
    middleware::auth_middleware::auth_middleware,
    state::AppState,
};

pub fn router(state: AppState) -> Router<AppState> {
    // Giới hạn body = giới hạn file + overhead của multipart (boundary, header).
    // Nếu để mặc định 2 MB, file đúng 2 MB sẽ bị chặn vì phần overhead.
    let body_limit = state.config.max_upload_bytes + 16 * 1024;

    Router::new()
        .route("/", post(upload_file))
        .route("/{file_name}", delete(delete_file))
        .layer(DefaultBodyLimit::max(body_limit))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware))
}