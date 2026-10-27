use axum::{
    Json,
    extract::{
        Path, State,
        rejection::{JsonRejection, PathRejection},
    },
};

use crate::{
    dto::user_dto::{
        UpdatePermissionsRequest, UserEnvelope, UserListEnvelope, UserResponse,
    },
    errors::AppError,
    middleware::auth_middleware::AuthUser,
    response::{ApiResponse, ErrorResponse},
    services::user_service,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Thông tin người dùng hiện tại", body = UserEnvelope),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse)
    )
)]
pub async fn get_me(auth: AuthUser) -> ApiResponse<UserResponse> {
    ApiResponse::success(
        "Thông tin người dùng hiện tại",
        UserResponse::from(&auth.0),
    )
}

#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Danh sách người dùng", body = UserListEnvelope),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin", body = ErrorResponse)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<ApiResponse<Vec<UserResponse>>, AppError> {
    auth.require_admin()?;

    let users = user_service::list_users(&state).await?;

    Ok(ApiResponse::success("Danh sách người dùng", users))
}

#[utoipa::path(
    put,
    path = "/api/users/{id}/permissions",
    tag = "users",
    security(("bearer_auth" = [])),
    request_body = UpdatePermissionsRequest,
    params(
        ("id" = i64, Path, description = "ID người dùng cần cập nhật quyền")
    ),
    responses(
        (status = 200, description = "Cập nhật quyền thành công", body = UserEnvelope),
        (status = 400, description = "Dữ liệu không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin hoặc tự thay đổi quyền của mình", body = ErrorResponse),
        (status = 404, description = "Không tìm thấy người dùng", body = ErrorResponse)
    )
)]
pub async fn update_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
    path: Result<Path<i64>, PathRejection>,
    payload: Result<Json<UpdatePermissionsRequest>, JsonRejection>,
) -> Result<ApiResponse<UserResponse>, AppError> {
    auth.require_admin()?;

    let Path(user_id) = path?;
    let Json(payload) = payload?;

    let user = user_service::update_permissions(&state, &auth.0, user_id, payload).await?;

    Ok(ApiResponse::success("Cập nhật quyền thành công", user))
}