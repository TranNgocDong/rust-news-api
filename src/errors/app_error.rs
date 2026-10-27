
use axum::{
    Json, 
    extract::rejection::{JsonRejection, PathRejection, QueryRejection}, 
    http::StatusCode, 
    response::{IntoResponse, Response},
};

use crate::response::ApiResponse;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    ServiceUnavailable(String),
    Internal(String),
    UnAuthorized(String),
    Conflict(String),
    Forbidden(String),

}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::ServiceUnavailable(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
            Self::UnAuthorized(message) => (StatusCode::UNAUTHORIZED, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            Self::Internal(detail) => {
                eprintln!("Lỗi nội bộ: {detail}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Đã xảy ra lỗi, vui lòng thử lại sau".to_string(),
                )
            }
        };

        (status, Json(ApiResponse::<()>::error(message))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound("Không tìm thấy dữ liệu".to_string()),
            other => Self::Internal(format!("Lỗi database: {other}")),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::BadRequest(
            format!("Lỗi khi phân tích JSON: {}", rejection.body_text())
        )
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        Self::BadRequest(
            format!("Lỗi khi phân tích đường dẫn: {}", rejection.to_string())
        )
    }
}
impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        Self::BadRequest(format!(
            "Lỗi khi phân tích query: {}",
            rejection.body_text()
        ))
    }
}