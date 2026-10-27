use axum::{
    extract::{
        Multipart, Path, State,
        multipart::MultipartRejection,
        rejection::PathRejection,
    },
    http::StatusCode,
};

use crate::{
    dto::file_dto::{FileEnvelope, UploadFileRequest, UploadFileResponse},
    errors::AppError,
    middleware::auth_middleware::AuthUser,
    response::{ApiResponse, ErrorResponse},
    services::file_service,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/files",
    tag = "files",
    security(("bearer_auth" = [])),
    request_body(content = UploadFileRequest, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Upload file thành công", body = FileEnvelope),
        (status = 400, description = "File rỗng, sai loại hoặc thiếu field", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin/manager", body = ErrorResponse),
        (status = 413, description = "File quá lớn", body = ErrorResponse)
    )
)]
pub async fn upload_file(
    State(state): State<AppState>,
    auth: AuthUser,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, ApiResponse<UploadFileResponse>), AppError> {
    if !auth.0.can_manage_news() {
        return Err(AppError::Forbidden(
            "Chỉ admin hoặc manager mới được upload file".to_string(),
        ));
    }

    let multipart = multipart?;

    let data = file_service::save_upload(&state, multipart).await?;

    Ok((
        StatusCode::CREATED,
        ApiResponse::success("Upload file thành công", data),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/files/{file_name}",
    tag = "files",
    security(("bearer_auth" = [])),
    params(
        ("file_name" = String, Path, description = "Tên file do server trả về khi upload")
    ),
    responses(
        (status = 204, description = "Xóa file thành công"),
        (status = 400, description = "Tên file không hợp lệ", body = ErrorResponse),
        (status = 401, description = "Chưa đăng nhập hoặc token không hợp lệ", body = ErrorResponse),
        (status = 403, description = "Không phải admin/manager", body = ErrorResponse),
        (status = 404, description = "Không tìm thấy file", body = ErrorResponse)
    )
)]
pub async fn delete_file(
    State(state): State<AppState>,
    auth: AuthUser,
    path: Result<Path<String>, PathRejection>,
) -> Result<StatusCode, AppError> {
    if !auth.0.can_manage_news() {
        return Err(AppError::Forbidden(
            "Chỉ admin hoặc manager mới được xóa file".to_string(),
        ));
    }

    let Path(file_name) = path?;

    file_service::delete_file(&state, &file_name).await?;

    Ok(StatusCode::NO_CONTENT)
}