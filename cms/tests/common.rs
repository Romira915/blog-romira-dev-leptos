#![allow(dead_code)]

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// テスト用のデータベースプールを作成
/// 各テストで独自のプールを持つことで、テスト間の干渉を防ぐ
pub async fn create_test_pool() -> PgPool {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// テスト用の一意なプレフィックスを生成
pub fn unique_prefix() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// テスト用のカテゴリを作成する
pub async fn create_test_category(pool: &PgPool, name: &str, slug: &str) -> uuid::Uuid {
    sqlx::query_scalar!(
        r#"
        INSERT INTO categories (name, slug)
        VALUES ($1, $2)
        RETURNING id
        "#,
        name,
        slug
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test category")
}

/// テスト用の下書き記事を直接DBに挿入する（Repository経由ではなく）
pub async fn insert_draft_article_directly(
    pool: &PgPool,
    slug: &str,
    title: &str,
    body: &str,
    description: Option<&str>,
) -> uuid::Uuid {
    sqlx::query_scalar!(
        r#"
        INSERT INTO draft_articles (slug, title, body, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        slug,
        title,
        body,
        description
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert draft article directly")
}

/// テスト用の公開記事を直接DBに挿入する
pub async fn insert_published_article_directly(
    pool: &PgPool,
    slug: &str,
    title: &str,
    body: &str,
    description: Option<&str>,
    published_at: chrono::NaiveDateTime,
) -> uuid::Uuid {
    sqlx::query_scalar!(
        r#"
        INSERT INTO published_articles (slug, title, body, description, published_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        slug,
        title,
        body,
        description,
        published_at as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert published article directly")
}

/// 下書き記事にカテゴリを関連付ける
pub async fn link_draft_article_category(
    pool: &PgPool,
    article_id: uuid::Uuid,
    category_id: uuid::Uuid,
) {
    sqlx::query!(
        r#"
        INSERT INTO draft_article_categories (article_id, category_id)
        VALUES ($1, $2)
        "#,
        article_id,
        category_id
    )
    .execute(pool)
    .await
    .expect("Failed to link draft article category");
}

/// 公開記事にカテゴリを関連付ける
pub async fn link_published_article_category(
    pool: &PgPool,
    article_id: uuid::Uuid,
    category_id: uuid::Uuid,
) {
    sqlx::query!(
        r#"
        INSERT INTO published_article_categories (article_id, category_id)
        VALUES ($1, $2)
        "#,
        article_id,
        category_id
    )
    .execute(pool)
    .await
    .expect("Failed to link published article category");
}
