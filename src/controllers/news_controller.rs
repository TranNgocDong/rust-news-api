use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
};

use crate::{
    dto::news_dto::{
        CreateNewsRequest, NewsEnvelope, NewsListData, NewsListEnvelope, NewsListQuery,
        NewsResponse, UpdateNewsRequest,
    },
    errors::AppError,
    middleware::auth_middleware::{AuthUser, OptionalAuthUser},
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
) -> Result<ApiResponse<NewsResponse>, AppError> {
    let Path(id) = path?;

    let news = news_service::get_news_by_id(&state, id).await?;

    Ok(ApiResponse::success("Chi tiết tin tức", news))
}

#[utoipa::path(
    post,
    path = "/api/news",
    tag = "news",
    security(("bearer_auth" = [])),
    request_body = CreateNewsRequest,
    responses(
        (status = 201, description = "Tạo tin tức thành công", body = NewsEnvelope),
        (status = 400, description = "Dữ liệu không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin/manager", body = ErrorResponse)
    )
)]
pub async fn create_news(
    State(state): State<AppState>,
    auth: AuthUser,
    payload: Result<Json<CreateNewsRequest>, JsonRejection>,
) -> Result<(StatusCode, ApiResponse<NewsResponse>), AppError> {
    auth.require_news_manager()?;

    let Json(payload) = payload?;

    let news = news_service::create_news(&state, auth.0.id, payload).await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::success("Tạo tin tức thành công", news),
    ))
}

#[utoipa::path(
    put,
    path = "/api/news/{id}",
    tag = "news",
    security(("bearer_auth" = [])),
    request_body = UpdateNewsRequest,
    params(
        ("id" = i64, Path, description = "ID tin tức cần cập nhật")
    ),
    responses(
        (status = 200, description = "Cập nhật tin tức thành công", body = NewsEnvelope),
        (status = 400, description = "Dữ liệu không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin/manager", body = ErrorResponse),
        (status = 404, description = "Không tìm thấy tin tức", body = ErrorResponse)
    )
)]
pub async fn update_news(
    State(state): State<AppState>,
    auth: AuthUser,
    path: Result<Path<i64>, PathRejection>,
    payload: Result<Json<UpdateNewsRequest>, JsonRejection>,
) -> Result<ApiResponse<NewsResponse>, AppError> {
    auth.require_news_manager()?;

    let Path(id) = path?;
    let Json(payload) = payload?;

    let news = news_service::update_news(&state, id, payload).await?;

    Ok(ApiResponse::success("Cập nhật tin tức thành công", news))
}

#[utoipa::path(
    delete,
    path = "/api/news/{id}",
    tag = "news",
    security(("bearer_auth" = [])),
    params(
        ("id" = i64, Path, description = "ID tin tức cần xóa")
    ),
    responses(
        (status = 204, description = "Xóa tin tức thành công"),
        (status = 400, description = "ID không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin/manager", body = ErrorResponse),
        (status = 404, description = "Không tìm thấy tin tức", body = ErrorResponse)
    )
)]
pub async fn delete_news(
    State(state): State<AppState>,
    auth: AuthUser,
    path: Result<Path<i64>, PathRejection>,
) -> Result<StatusCode, AppError> {
    auth.require_news_manager()?;

    let Path(id) = path?;

    news_service::delete_news(&state, id).await?;

    Ok(StatusCode::NO_CONTENT)
}