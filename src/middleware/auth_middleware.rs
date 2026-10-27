use axum::{
    extract::{FromRequestParts, Request, State},
    http::{HeaderMap, header, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::{
    errors::AppError,
    models::user::User,
    services::auth_service,
    state::AppState,
};

#[derive(Clone)]
pub struct AuthUser(pub User);

impl AuthUser {
    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.0.is_admin() {
            Ok(())
        } else {
            Err(AppError::Forbidden(
                "Chỉ admin mới có quyền thực hiện thao tác này".to_string(),
            ))
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| {
                AppError::UnAuthorized("Chưa xác thực hoặc thiếu middleware".to_string())
            })
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = {
        let headers = request.headers();
        extract_bearer_token(headers)?.to_string()
    };

    let claims = auth_service::decode_token(&token, &state.config.jwt_secret)?;

    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| AppError::UnAuthorized("Token không hợp lệ".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, role, is_manager, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        AppError::UnAuthorized("Người dùng không tồn tại hoặc đã bị xóa".to_string())
    })?;

    request.extensions_mut().insert(AuthUser(user));

    Ok(next.run(request).await)
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::UnAuthorized("Thiếu Authorization header".to_string()))?;

    let (scheme, token) = value.split_once(' ').ok_or_else(|| {
        AppError::UnAuthorized("Authorization phải có dạng: Bearer <token>".to_string())
    })?;

    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(AppError::UnAuthorized(
            "Authorization phải dùng Bearer token".to_string(),
        ));
    }

    let token = token.trim();

    if token.is_empty() {
        return Err(AppError::UnAuthorized(
            "Token không được để trống".to_string(),
        ));
    }

    Ok(token)
}