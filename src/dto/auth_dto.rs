use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::dto::user_dto::UserResponse;



#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}


#[allow(dead_code)]
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthEnvelope {
    pub success: bool,
    pub message: String,
    pub data: AuthResponse,
}