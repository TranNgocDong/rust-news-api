use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadFileResponse {
    /// Tên file đã được server sinh ra
    pub file_name: String,
    /// Đường dẫn tương đối (chưa serve tĩnh)
    pub path: String,
    /// Kích thước tính bằng byte
    pub size: u64,
    /// MIME type đã kiểm tra
    pub content_type: String,
}

#[allow(dead_code)] // Chỉ dùng làm schema response trong Swagger
#[derive(Debug, Serialize, ToSchema)]
pub struct FileEnvelope {
    pub success: bool,
    pub message: String,
    pub data: UploadFileResponse,
}

#[allow(dead_code)] // Chỉ dùng làm schema request trong Swagger
#[derive(Debug, ToSchema)]
pub struct UploadFileRequest {
    /// Chọn file ảnh (field name phải là "file")
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}