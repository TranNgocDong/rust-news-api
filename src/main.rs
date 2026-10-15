mod config;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use config::Config;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let app = Router::new().route("/", get(home));

    let address = config.server_address();

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Không thể khởi động server");

    println!("Server đang chạy tại http://{}", address);

    axum::serve(listener, app).await.expect("Server gặp lỗi");
}

async fn home() -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "News API is running"
    }))
}
