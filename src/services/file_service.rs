use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Multipart, multipart::MultipartError},
    http::StatusCode,
};
use tokio::io::AsyncWriteExt;

use crate::{dto::file_dto::UploadFileResponse, errors::AppError, state::AppState};

pub async fn save_upload(
    state: &AppState,
    mut multipart: Multipart,
) -> Result<UploadFileResponse, AppError> {
    tokio::fs::create_dir_all(&state.config.upload_dir)
        .await
        .map_err(|error| AppError::Internal(format!("Không tạo được thư mục upload: {error}")))?;

    let mut field = multipart
        .next_field()
        .await
        .map_err(map_multipart_error)?
        .ok_or_else(|| AppError::BadRequest("Thiếu field 'file'".to_string()))?;

    if field.name() != Some("file") {
        return Err(AppError::BadRequest(
            "Field phải có tên là 'file'".to_string(),
        ));
    }

    // Chỉ tin content-type, KHÔNG dùng extension của client để đặt tên file.
    let content_type = field
        .content_type()
        .map(|value| value.split(';').next().unwrap_or("").trim().to_lowercase())
        .ok_or_else(|| AppError::BadRequest("Thiếu Content-Type của file".to_string()))?;

    let extension = extension_from_content_type(&content_type).ok_or_else(|| {
        AppError::BadRequest("Chỉ chấp nhận ảnh jpeg, png, webp hoặc gif".to_string())
    })?;

    // Đọc theo chunk để không nạp cả file quá lớn vào RAM trước khi kiểm tra.
    let mut data: Vec<u8> = Vec::new();

    while let Some(chunk) = field.chunk().await.map_err(map_multipart_error)? {
        if data.len() + chunk.len() > state.config.max_upload_bytes {
            return Err(AppError::PayloadTooLarge(format!(
                "File vượt quá dung lượng tối đa {} bytes",
                state.config.max_upload_bytes
            )));
        }
        data.extend_from_slice(&chunk);
    }

    if data.is_empty() {
        return Err(AppError::BadRequest("File rỗng".to_string()));
    }

    let file_name = write_new_file(&state.config.upload_dir, extension, &data).await?;

    Ok(UploadFileResponse {
        path: format!("/uploads/{file_name}"),
        file_name,
        size: data.len() as u64,
        content_type,
    })
}

pub async fn delete_file(state: &AppState, file_name: &str) -> Result<(), AppError> {
    validate_file_name(file_name)?;

    let path = std::path::Path::new(&state.config.upload_dir).join(file_name);

    match tokio::fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Err(AppError::NotFound("Không tìm thấy file".to_string()))
        }
        Err(error) => Err(AppError::Internal(format!("Không xóa được file: {error}"))),
    }
}

fn extension_from_content_type(content_type: &str) -> Option<&'static str> {
    match content_type {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        "image/gif" => Some("gif"),
        _ => None,
    }
}

/// Sinh tên mới + tạo file với cờ create_new (không ghi đè).
async fn write_new_file(
    upload_dir: &str,
    extension: &str,
    data: &[u8],
) -> Result<String, AppError> {
    for attempt in 0..5 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| AppError::Internal(format!("Lỗi thời gian hệ thống: {error}")))?
            .as_nanos();

        let file_name = format!("{nanos}_{attempt}.{extension}");
        let path = std::path::Path::new(upload_dir).join(&file_name);

        let opened = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await;

        match opened {
            Ok(mut file) => {
                if let Err(error) = file.write_all(data).await {
                    // Ghi lỗi giữa chừng thì dọn file dở, tránh rác trong uploads/.
                    let _ = tokio::fs::remove_file(&path).await;
                    return Err(AppError::Internal(format!("Không ghi được file: {error}")));
                }

                file.flush()
                    .await
                    .map_err(|error| AppError::Internal(format!("Không ghi được file: {error}")))?;

                return Ok(file_name);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(AppError::Internal(format!("Không tạo được file: {error}")));
            }
        }
    }

    Err(AppError::Internal(
        "Không sinh được tên file duy nhất".to_string(),
    ))
}

/// Chặn path traversal và ký tự lạ trước khi ghép vào UPLOAD_DIR.
fn validate_file_name(file_name: &str) -> Result<(), AppError> {
    let invalid = file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
        || file_name.starts_with('.')
        || !file_name.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '.'
                || character == '_'
                || character == '-'
        });

    if invalid {
        return Err(AppError::BadRequest("Tên file không hợp lệ".to_string()));
    }

    Ok(())
}

fn map_multipart_error(error: MultipartError) -> AppError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::PayloadTooLarge("Dữ liệu upload vượt quá giới hạn".to_string())
    } else {
        AppError::BadRequest(format!("Lỗi multipart: {}", error.body_text()))
    }
}
