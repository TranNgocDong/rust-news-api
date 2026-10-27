use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::user::User;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub is_manager: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            is_manager: user.is_manager,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePermissionsRequest {
    pub role: String,
    pub is_manager: bool,
}

#[allow(dead_code)] // Chỉ dùng làm schema response trong Swagger
#[derive(Debug, Serialize, ToSchema)]
pub struct UserEnvelope {
    pub success: bool,
    pub message: String,
    pub data: UserResponse,
}

#[allow(dead_code)] // Chỉ dùng làm schema response trong Swagger
#[derive(Debug, Serialize, ToSchema)]
pub struct UserListEnvelope {
    pub success: bool,
    pub message: String,
    pub data: Vec<UserResponse>,
}
