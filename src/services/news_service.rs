use sqlx::AssertSqlSafe;

use crate::{
    dto::news_dto::{
        CreateNewsRequest, NewsListData, NewsListQuery, NewsResponse, UpdateNewsRequest,
    },
    errors::AppError,
    models::news::News,
    state::AppState,
};
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

// Bộ lọc dùng chung cho câu SELECT và câu COUNT.
// $1 = pattern tìm kiếm (NULL nếu không tìm).
const SEARCH_FILTER: &str = "( $1::text IS NULL OR title ILIKE $1 OR content ILIKE $1 )";

pub async fn list_news(
    state: &AppState,
    query: NewsListQuery,
) -> Result<NewsListData, AppError> {
    let page = query.page.unwrap_or(DEFAULT_PAGE);
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);

    if page == 0 {
        return Err(AppError::BadRequest("page phải lớn hơn hoặc bằng 1".to_string()));
    }

    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(AppError::BadRequest(format!(
            "limit phải nằm trong khoảng 1..={MAX_LIMIT}"
        )));
    }

    // Whitelist: chỉ 3 giá trị cố định này được phép lọt vào SQL.
    // Nhờ vậy không cần nối chuỗi từ input người dùng -> không SQL injection.
    let sort_column = match query.sort.as_deref() {
        None | Some("created_at") => "created_at",
        Some("updated_at") => "updated_at",
        Some("title") => "title",
        Some(_) => {
            return Err(AppError::BadRequest(
                "sort chỉ được là created_at, updated_at hoặc title".to_string(),
            ));
        }
    };

    let ascending = match query.order.as_deref() {
        None | Some("desc") => false,
        Some("asc") => true,
        Some(_) => {
            return Err(AppError::BadRequest(
                "order chỉ được là asc hoặc desc".to_string(),
            ));
        }
    };

    let direction = if ascending { "ASC" } else { "DESC" };
    let offset = i64::from(page - 1) * i64::from(limit);

    // q rỗng hoặc toàn khoảng trắng coi như không tìm kiếm.
    let pattern = query
        .q
        .map(|keyword| keyword.trim().to_string())
        .filter(|keyword| !keyword.is_empty())
        .map(|keyword| format!("%{keyword}%"));

    let list_sql = format!(
        r#"
        SELECT id, title, content, author_id, image_url, created_at, updated_at
        FROM news
        WHERE {SEARCH_FILTER}
        ORDER BY {sort_column} {direction}
        LIMIT $2 OFFSET $3
        "#
    );

    // sqlx 0.9 yêu cầu SQL phải là &'static str (trait SqlSafeStr).
    // Ở đây chuỗi được ghép động nhưng chỉ từ whitelist cố định ở trên,
    // không chứa input người dùng, nên AssertSqlSafe là hợp lý.
    let items = sqlx::query_as::<_, News>(AssertSqlSafe(list_sql))
        .bind(&pattern)
        .bind(i64::from(limit))
        .bind(offset)
        .fetch_all(&state.pool)
        .await?;

    let count_sql = format!("SELECT COUNT(*) FROM news WHERE {SEARCH_FILTER}");

    let total = sqlx::query_scalar::<_, i64>(AssertSqlSafe(count_sql))
        .bind(&pattern)
        .fetch_one(&state.pool)
        .await?;

    Ok(NewsListData {
        items: items.iter().map(NewsResponse::from).collect(),
        page,
        limit,
        total,
    })
}

pub async fn get_news_by_id(state: &AppState, id: i64) -> Result<NewsResponse, AppError> {
    let news = sqlx::query_as::<_, News>(
        r#"
        SELECT id, title, content, author_id, image_url, created_at, updated_at
        FROM news
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Không tìm thấy tin tức".to_string()))?;

    Ok(NewsResponse::from(&news))
}


pub async fn create_news(
    state: &AppState,
    author_id: i64,
    payload: CreateNewsRequest,
) -> Result<NewsResponse, AppError> {
    let (title, content, image_url) =
        normalize_news_input(payload.title, payload.content, payload.image_url)?;

    let news = sqlx::query_as::<_, News>(
        r#"
        INSERT INTO news (title, content, author_id, image_url)
        VALUES ($1, $2, $3, $4)
        RETURNING id, title, content, author_id, image_url, created_at, updated_at
        "#,
    )
    .bind(&title)
    .bind(&content)
    .bind(author_id)
    .bind(&image_url)
    .fetch_one(&state.pool)
    .await?;

    Ok(NewsResponse::from(&news))
}

pub async fn update_news(
    state: &AppState,
    id: i64,
    payload: UpdateNewsRequest,
) -> Result<NewsResponse, AppError> {
    let (title, content, image_url) =
        normalize_news_input(payload.title, payload.content, payload.image_url)?;

    let news = sqlx::query_as::<_, News>(
        r#"
        UPDATE news
        SET title = $1,
            content = $2,
            image_url = $3,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, title, content, author_id, image_url, created_at, updated_at
        "#,
    )
    .bind(&title)
    .bind(&content)
    .bind(&image_url)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Không tìm thấy tin tức".to_string()))?;

    Ok(NewsResponse::from(&news))
}

pub async fn delete_news(state: &AppState, id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM news WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Không tìm thấy tin tức".to_string()));
    }

    Ok(())
}

/// Chuẩn hoá + validate input dùng chung cho create và update.
fn normalize_news_input(
    title: String,
    content: String,
    image_url: Option<String>,
) -> Result<(String, String, Option<String>), AppError> {
    let title = title.trim().to_string();

    if title.is_empty() {
        return Err(AppError::BadRequest("title không được để trống".to_string()));
    }

    // DB là VARCHAR(255) tính theo ký tự, nên đếm bằng chars() chứ không phải len() (byte).
    if title.chars().count() > 255 {
        return Err(AppError::BadRequest(
            "title không được dài quá 255 ký tự".to_string(),
        ));
    }

    let content = content.trim().to_string();

    if content.is_empty() {
        return Err(AppError::BadRequest(
            "content không được để trống".to_string(),
        ));
    }

    // Chuỗi rỗng hoặc toàn khoảng trắng coi như không có ảnh.
    let image_url = image_url
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty());

    if let Some(url) = &image_url {
        if url.chars().count() > 2048 {
            return Err(AppError::BadRequest(
                "image_url không được dài quá 2048 ký tự".to_string(),
            ));
        }
    }

    Ok((title, content, image_url))
}