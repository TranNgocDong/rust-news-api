use axum::{
    extract::{
        Path, Query, State,
        rejection::{PathRejection, QueryRejection},
    },
};

use crate::{
    dto::news_dto::{
        NewsEnvelope, NewsListData, NewsListEnvelope, NewsListQuery,
    },
    errors::AppError,
    middleware::auth_middleware::OptionalAuthUser,
    response::{ApiResponse, ErrorResponse},
    services::news_service,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/news",
    tag = "news",
    security(("bearer_auth" = [])),
    params(NewsListQuery),
    responses(
        (status = 200, description = "Danh sách tin tức", body = NewsListEnvelope),
        (status = 400, description = "Query không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Dùng sort tùy chỉnh nhưng chưa đăng nhập, hoặc token không hợp lệ", body = ErrorResponse)
    )
)]
pub async fn list_news(
    State(state): State<AppState>,
    auth: OptionalAuthUser,
    params: Result<Query<NewsListQuery>, QueryRejection>,
) -> Result<ApiResponse<NewsListData>, AppError> {
    let Query(query) = params?;

    // Chỉ chặn sort khi CHƯA đăng nhập. Đã có token thì role nào cũng được sort.
    let wants_custom_sort = query.sort.is_some() || query.order.is_some();

    if wants_custom_sort && auth.0.is_none() {
        return Err(AppError::UnAuthorized(
            "Vui lòng đăng nhập để dùng sort tùy chỉnh".to_string(),
        ));
    }

    let data = news_service::list_news(&state, query).await?;

    Ok(ApiResponse::success("Danh sách tin tức", data))
}

#[utoipa::path(
    get,
    path = "/api/news/{id}",
    tag = "news",
    params(
        ("id" = i64, Path, description = "ID tin tức")
    ),
    responses(
        (status = 200, description = "Chi tiết tin tức", body = NewsEnvelope),
        (status = 400, description = "ID không hợp lệ", body = ErrorResponse),
        (status = 404, description = "Không tìm thấy tin tức", body = ErrorResponse)
    )
)]
pub async fn get_news(
    State(state): State<AppState>,
    path: Result<Path<i64>, PathRejection>,
) -> Result<ApiResponse<crate::dto::news_dto::NewsResponse>, AppError> {
    let Path(id) = path?;

    let news = news_service::get_news_by_id(&state, id).await?;

    Ok(ApiResponse::success("Chi tiết tin tức", news))
}