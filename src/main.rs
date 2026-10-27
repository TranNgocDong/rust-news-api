mod config;
mod errors;
mod response;
mod state;
#[allow(dead_code)]
mod models;
mod dto;
mod controllers;
mod routes;
mod services;
mod docs;
mod middleware;

use docs::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use axum::{
    Router,
    extract::State,
    routing::get,
};
use serde_json::{Value, json};

use config::{Config, database};
use errors::AppError;
use response::ApiResponse;
use state::AppState;

#[tokio::main]
async fn main() {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("Lỗi cấu hình: {message}");
            std::process::exit(1);
        }
    };

    let pool = match database::create_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Không thể kết nối PostgreSQL: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = sqlx::query("SELECT 1").execute(&pool).await {
        eprintln!("Truy vấn kiểm tra database thất bại: {error}");
        std::process::exit(1);
    }

    println!("Đã kết nối PostgreSQL.");
    if let Err(error) = sqlx::migrate!().run(&pool).await {
        eprintln!("Lỗi khi chạy migration: {error}");
        std::process::exit(1);
    }
    println!("Đã chạy migration thành công.");
    print!("Cấu hình JWT {} ký tự | upload {} | tối đa {} \n ", config.jwt_secret.len(), config.upload_dir, config.max_upload_bytes);

    let address = config.server_address();
    let app_state = AppState::new(pool, config);

    let app = Router::<AppState>::new()
        .route("/", get(home))
        .route("/health", get(health))
        .merge(routes::router(app_state.clone()))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .fallback(not_found)
        .with_state(app_state);

    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Không thể mở cổng {address}: {error}");
            std::process::exit(1);
        }
    };

    println!("Server đang chạy tại http://{address}");

    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("Server gặp lỗi: {error}");
        std::process::exit(1);
    }
}

async fn home() -> ApiResponse<()> {
    ApiResponse::ok("News API is running")
}

async fn health(State(state): State<AppState>) -> Result<ApiResponse<Value>, AppError> {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => Ok(ApiResponse::success(
            "API và database hoạt động bình thường",
            json!({
                "database": "up",
                "server": state.config.server_address()
            }),
        )),
        Err(error) => {
            eprintln!("Health check: database gặp lỗi: {error}");
            Err(AppError::ServiceUnavailable(
                "Database không khả dụng".to_string(),
            ))
        }
    }
}

async fn not_found() -> AppError {
    AppError::NotFound("Không tìm thấy endpoint".to_string())
}