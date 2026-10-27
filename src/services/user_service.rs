use crate::{
    dto::user_dto::{UpdatePermissionsRequest, UserResponse},
    errors::AppError,
    models::user::{ROLE_ADMIN, ROLE_USER, User},
    state::AppState,
};

pub async fn list_users(state: &AppState) -> Result<Vec<UserResponse>, AppError> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, role, is_manager, created_at
        FROM users
        ORDER BY id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(users.iter().map(UserResponse::from).collect())
}

pub async fn update_permissions(
    state: &AppState,
    actor: &User,
    target_id: i64,
    payload: UpdatePermissionsRequest,
) -> Result<UserResponse, AppError> {
    if actor.id == target_id {
        return Err(AppError::Forbidden(
            "Admin không thể tự thay đổi quyền của chính mình".to_string(),
        ));
    }

    let role = payload.role.trim().to_lowercase();

    if role != ROLE_ADMIN && role != ROLE_USER {
        return Err(AppError::BadRequest(
            "role chỉ được là 'admin' hoặc 'user'".to_string(),
        ));
    }

    // admin luôn phải có is_manager = true
    let is_manager = role == ROLE_ADMIN || payload.is_manager;

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET role = $1, is_manager = $2
        WHERE id = $3
        RETURNING id, username, password_hash, role, is_manager, created_at
        "#,
    )
    .bind(&role)
    .bind(is_manager)
    .bind(target_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Không tìm thấy người dùng".to_string()))?;

    Ok(UserResponse::from(&user))
}
