use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::models::news::News;

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&News> for NewsResponse {
    fn from(news: &News) -> Self {
        Self {
            id: news.id,
            title: news.title.clone(),
            content: news.content.clone(),
            author_id: news.author_id,
            image_url: news.image_url.clone(),
            created_at: news.created_at,
            updated_at: news.updated_at,
        }
    }
}

/// Query param của GET /api/news.
/// `IntoParams` để utoipa đưa từng field thành query parameter trong Swagger.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct NewsListQuery {
    /// Từ khóa tìm trong title hoặc content (không phân biệt hoa thường)
    pub q: Option<String>,
    /// Trang hiện tại, bắt đầu từ 1 (mặc định 1)
    pub page: Option<u32>,
    /// Số bài mỗi trang, 1..=100 (mặc định 10)
    pub limit: Option<u32>,
    /// Cột sắp xếp: created_at | updated_at | title (mặc định created_at)
    pub sort: Option<String>,
    /// Chiều sắp xếp: asc | desc (mặc định desc)
    pub order: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NewsListData {
    pub items: Vec<NewsResponse>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

#[allow(dead_code)] // Chỉ dùng làm schema response trong Swagger
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsEnvelope {
    pub success: bool,
    pub message: String,
    pub data: NewsResponse,
}

#[allow(dead_code)] // Chỉ dùng làm schema response trong Swagger
#[derive(Debug, Serialize, ToSchema)]
pub struct NewsListEnvelope {
    pub success: bool,
    pub message: String,
    pub data: NewsListData,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNewsRequest {
    /// Tiêu đề, 1..=255 ký tự
    #[schema(example = "Rust va Axum", max_length = 255)]
    pub title: String,
    /// Nội dung, không được rỗng
    #[schema(example = "Noi dung bai viet")]
    pub content: String,
    /// URL ảnh, không bắt buộc
    #[schema(example = "https://example.com/anh.jpg")]
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNewsRequest {
    /// Tiêu đề, 1..=255 ký tự
    #[schema(example = "Tieu de moi", max_length = 255)]
    pub title: String,
    /// Nội dung, không được rỗng
    #[schema(example = "Noi dung moi")]
    pub content: String,
    /// URL ảnh; bỏ trống hoặc null nghĩa là xoá ảnh
    pub image_url: Option<String>,
}
