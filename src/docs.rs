use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::{
    controllers::{auth_controller, news_controller, user_controller},
    dto::{
        auth_dto::{AuthEnvelope, AuthResponse, LoginRequest, RegisterRequest},
        news_dto::{
            NewsEnvelope, NewsListData, NewsListEnvelope, NewsResponse, CreateNewsRequest, UpdateNewsRequest
        },
        user_dto::{
            UpdatePermissionsRequest, UserEnvelope, UserListEnvelope, UserResponse,
        },
    },
    response::ErrorResponse,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(
        title = "News API",
        version = "0.1.0",
        description = "Backend quản lý tin tức bằng Rust, Axum và PostgreSQL."
    ),
    paths(
        auth_controller::register,
        auth_controller::login,
        user_controller::get_me,
        user_controller::list_users,
        user_controller::update_permissions,
        news_controller::list_news,
        news_controller::get_news,
        news_controller::create_news,
        news_controller::update_news,
        news_controller::delete_news
    ),
    components(schemas(
        RegisterRequest,
        LoginRequest,
        AuthResponse,
        AuthEnvelope,
        UserResponse,
        UserEnvelope,
        UserListEnvelope,
        UpdatePermissionsRequest,
        NewsResponse,
        NewsListData,
        NewsEnvelope,
        NewsListEnvelope,
        ErrorResponse,
        CreateNewsRequest,
        UpdateNewsRequest
    )),
    tags(
        (name = "auth", description = "Đăng ký và đăng nhập"),
        (name = "users", description = "Thông tin người dùng và phân quyền"),
        (name = "news", description = "Đọc, tìm kiếm và sắp xếp tin tức")
    )
)]
pub struct ApiDoc;