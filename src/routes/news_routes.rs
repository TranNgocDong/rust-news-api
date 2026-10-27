use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::{
    controllers::news_controller::{create_news, delete_news, get_news, list_news, update_news},
    middleware::auth_middleware::auth_middleware,
    state::AppState,
};

pub fn router(state: AppState) -> Router<AppState> {
    // Public: ai cũng đọc/tìm kiếm được.
    let public: Router<AppState> = Router::new()
        .route("/", get(list_news))
        .route("/{id}", get(get_news));

    // Private: middleware xác thực chỉ phủ nhóm route này.
    let private: Router<AppState> = Router::new()
        .route("/", post(create_news))
        .route("/{id}", put(update_news).delete(delete_news))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    public.merge(private)
}
