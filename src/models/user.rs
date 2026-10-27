use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_USER: &str = "user";

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub is_manager: bool,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN
    }

    pub fn can_manage_news(&self) -> bool {
        self.is_admin() || self.is_manager
    }

    pub fn can_manage_users(&self) -> bool {
        self.is_admin()
    }
}
