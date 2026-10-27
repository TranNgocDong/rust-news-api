use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
//use sqlx::error::DatabaseError;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};

use crate::{
    dto::{
        auth_dto::{AuthResponse, LoginRequest, RegisterRequest},
        user_dto::UserResponse,
    },
    errors::AppError,
    models::user::User,
    state::AppState,
};

const TOKEN_TTL_HOURS: i64 = 24;
const TOKEN_TTL_SECONDS: i64 = TOKEN_TTL_HOURS * 3600;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn register(
    state: &AppState,
    payload: RegisterRequest,
) -> Result<AuthResponse, AppError> {
    let username = normalize_username(&payload.username)?;
    validate_password(&payload.password)?;

    let password = payload.password;

    let password_hash =
        tokio::task::spawn_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
            .await
            .map_err(|error| AppError::Internal(format!("Lỗi xử lý mật khẩu: {error}")))?
            .map_err(|error| AppError::Internal(format!("Lỗi hash mật khẩu: {error}")))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, password_hash, role, is_manager, created_at
        "#,
    )
    .bind(&username)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(map_insert_error)?;

    let token = create_token(user.id, &state.config.jwt_secret)?;

    Ok(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_TTL_SECONDS,
        user: UserResponse::from(&user),
    })
}

pub async fn login(state: &AppState, payload: LoginRequest) -> Result<AuthResponse, AppError> {
    let username = normalize_username(&payload.username)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, role, is_manager, created_at
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?;

    let Some(user) = user else {
        return Err(AppError::UnAuthorized(
            "Tên đăng nhập hoặc mật khẩu không đúng".to_string(),
        ));
    };

    let password = payload.password;
    let password_hash = user.password_hash.clone();

    let is_valid = tokio::task::spawn_blocking(move || bcrypt::verify(password, &password_hash))
        .await
        .map_err(|error| AppError::Internal(format!("Lỗi xử lý mật khẩu: {error}")))?
        .map_err(|error| AppError::Internal(format!("Lỗi xác thực mật khẩu: {error}")))?;

    if !is_valid {
        return Err(AppError::UnAuthorized(
            "Tên đăng nhập hoặc mật khẩu không đúng".to_string(),
        ));
    }

    let token = create_token(user.id, &state.config.jwt_secret)?;

    Ok(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_TTL_SECONDS,
        user: UserResponse::from(&user),
    })
}

fn normalize_username(raw: &str) -> Result<String, AppError> {
    let username = raw.trim().to_lowercase();

    if !(3..=50).contains(&username.len()) {
        return Err(AppError::BadRequest(
            "Username phải có từ 3 đến 50 ký tự".to_string(),
        ));
    }

    if !username
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AppError::BadRequest(
            "Username chỉ được gồm chữ, số và dấu gạch dưới".to_string(),
        ));
    }

    Ok(username)
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 6 {
        return Err(AppError::BadRequest(
            "Mật khẩu phải có ít nhất 6 ký tự".to_string(),
        ));
    }

    if password.len() > 72 {
        return Err(AppError::BadRequest(
            "Mật khẩu không được dài quá 72 byte".to_string(),
        ));
    }

    Ok(())
}

fn map_insert_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return AppError::Conflict("Tên đăng nhập đã tồn tại".to_string());
        }
    }

    AppError::from(error)
}

fn create_token(user_id: i64, secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::hours(TOKEN_TTL_HOURS)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| AppError::Internal(format!("Lỗi tạo JWT: {error}")))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|error| {
        let message = if matches!(error.kind(), ErrorKind::ExpiredSignature) {
            "Token đã hết hạn"
        } else {
            "Token không hợp lệ"
        };

        AppError::UnAuthorized(message.to_string())
    })?;

    Ok(token_data.claims)
}
