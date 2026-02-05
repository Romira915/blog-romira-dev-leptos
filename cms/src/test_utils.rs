//! テスト用ユーティリティ関数
//!
//! unit tests と integration tests の両方で使用可能

use chrono::{NaiveDateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// 現在時刻をUTC NaiveDateTimeで取得
pub fn utc_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

/// テスト用カテゴリを作成
pub async fn create_test_category(pool: &PgPool, name: &str, slug: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO categories (name, slug) VALUES ($1, $2) RETURNING id"#,
        name,
        slug
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test category")
}

/// テスト用下書き記事を作成
pub async fn insert_draft_article(
    pool: &PgPool,
    slug: &str,
    title: &str,
    body: &str,
    description: Option<&str>,
) -> Uuid {
    let now = utc_now();
    sqlx::query_scalar!(
        r#"INSERT INTO draft_articles (slug, title, body, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5) RETURNING id"#,
        slug,
        title,
        body,
        description,
        now as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert draft article")
}

/// テスト用公開記事を作成
pub async fn insert_published_article(
    pool: &PgPool,
    slug: &str,
    title: &str,
    body: &str,
    description: Option<&str>,
    published_at: NaiveDateTime,
) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO published_articles (slug, title, body, description, published_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
        slug,
        title,
        body,
        description,
        published_at as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert published article")
}

/// 下書き記事とカテゴリを紐付け
pub async fn link_draft_article_category(pool: &PgPool, article_id: Uuid, category_id: Uuid) {
    sqlx::query!(
        "INSERT INTO draft_article_categories (article_id, category_id) VALUES ($1, $2)",
        article_id,
        category_id
    )
    .execute(pool)
    .await
    .expect("Failed to link draft article category");
}

/// 公開記事とカテゴリを紐付け
pub async fn link_published_article_category(pool: &PgPool, article_id: Uuid, category_id: Uuid) {
    sqlx::query!(
        "INSERT INTO published_article_categories (article_id, category_id) VALUES ($1, $2)",
        article_id,
        category_id
    )
    .execute(pool)
    .await
    .expect("Failed to link published article category");
}

/// テスト用画像を作成
pub async fn insert_test_image(
    pool: &PgPool,
    filename: &str,
    gcs_path: &str,
    mime_type: &str,
    size_bytes: i64,
) -> Uuid {
    let now = utc_now();
    sqlx::query_scalar!(
        r#"INSERT INTO images (filename, gcs_path, mime_type, size_bytes, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
        filename,
        gcs_path,
        mime_type,
        size_bytes,
        now as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test image")
}

/// テスト用画像を作成（サイズ指定あり）
pub async fn insert_test_image_with_dimensions(
    pool: &PgPool,
    filename: &str,
    gcs_path: &str,
    mime_type: &str,
    size_bytes: i64,
    width: i32,
    height: i32,
) -> Uuid {
    let now = utc_now();
    sqlx::query_scalar!(
        r#"INSERT INTO images (filename, gcs_path, mime_type, size_bytes, width, height, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#,
        filename,
        gcs_path,
        mime_type,
        size_bytes,
        width,
        height,
        now as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert test image with dimensions")
}
