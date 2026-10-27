use axum::{
    Json,
    extract::State,
    extract::rejection::JsonRejection,
    http::StatusCode,
};

use crate::{
    dto::auth_dto::{AuthEnvelope, AuthResponse, LoginRequest, RegisterRequest},
    errors::AppError,
    response::{ApiResponse, ErrorResponse},
    services::auth_service,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Đăng ký thành công", body = AuthEnvelope),
        (status = 400, description = "Dữ liệu không hợp lệ", body = ErrorResponse),
        (status = 409, description = "Tên đăng nhập đã tồn tại", body = ErrorResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    payload: Result<Json<RegisterRequest>, JsonRejection>,
) -> Result<(StatusCode, ApiResponse<AuthResponse>), AppError> {
    let Json(payload) = payload?;
    let data = auth_service::register(&state, payload).await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::success("Đăng ký thành công", data),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Đăng nhập thành công", body = AuthEnvelope),
        (status = 400, description = "Dữ liệu không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Sai tên đăng nhập hoặc mật khẩu", body = ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let Json(payload) = payload?;
    let data = auth_service::login(&state, payload).await?;

    Ok(ApiResponse::success("Đăng nhập thành công", data))
}