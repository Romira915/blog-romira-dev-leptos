//! Service層の統合テスト
//! Repository + Query を組み合わせた実際のビジネスロジックをテスト

use blog_romira_dev_cms::error::CmsError;
use blog_romira_dev_cms::models::ArticleListItem;
use blog_romira_dev_cms::services::{
    AdminArticleService, DraftArticleService, PublishedArticleService,
};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

fn parse_datetime(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
}

async fn create_test_category(pool: &PgPool, name: &str, slug: &str) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO categories (name, slug) VALUES ($1, $2) RETURNING id"#,
        name,
        slug
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test category")
}

async fn link_draft_article_category(pool: &PgPool, article_id: Uuid, category_id: Uuid) {
    sqlx::query!(
        "INSERT INTO draft_article_categories (article_id, category_id) VALUES ($1, $2)",
        article_id,
        category_id
    )
    .execute(pool)
    .await
    .expect("Failed to link draft article category");
}

async fn insert_published_article(
    pool: &PgPool,
    slug: &str,
    title: &str,
    body: &str,
    published_at: NaiveDateTime,
) -> Uuid {
    sqlx::query_scalar!(
        r#"INSERT INTO published_articles (slug, title, body, published_at) VALUES ($1, $2, $3, $4) RETURNING id"#,
        slug,
        title,
        body,
        published_at as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert published article")
}

async fn insert_draft_article(pool: &PgPool, slug: &str, title: &str, body: &str) -> Uuid {
    let now = chrono::Utc::now().naive_utc();
    sqlx::query_scalar!(
        r#"INSERT INTO draft_articles (slug, title, body, created_at, updated_at) VALUES ($1, $2, $3, $4, $4) RETURNING id"#,
        slug,
        title,
        body,
        now as _
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert draft article")
}

// ============================================================
// DraftArticleService tests
// ============================================================

#[sqlx::test]
async fn test_draft_service_create_and_fetch(pool: PgPool) {
    let id = DraftArticleService::create(
        &pool,
        "サービス経由タイトル",
        "service-slug",
        "サービス経由本文",
        Some("説明"),
    )
    .await
    .expect("Failed to create draft");

    let fetched = DraftArticleService::fetch_by_id(&pool, id)
        .await
        .expect("Failed to fetch by id")
        .expect("Article not found");

    assert_eq!(fetched.article.title, "サービス経由タイトル");
    assert_eq!(fetched.article.slug, "service-slug");
    assert_eq!(fetched.article.body, "サービス経由本文");
    assert_eq!(fetched.article.description, Some("説明".to_string()));
}

#[sqlx::test]
async fn test_draft_service_fetch_all(pool: PgPool) {
    DraftArticleService::create(&pool, "Title 1", "slug1", "Body 1", None)
        .await
        .expect("Failed to create draft 1");

    DraftArticleService::create(&pool, "Title 2", "slug2", "Body 2", None)
        .await
        .expect("Failed to create draft 2");

    let all = DraftArticleService::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    assert_eq!(all.len(), 2);
}

#[sqlx::test]
async fn test_draft_service_update(pool: PgPool) {
    let id = DraftArticleService::create(&pool, "Original", "original-slug", "Original Body", None)
        .await
        .expect("Failed to create draft");

    DraftArticleService::update(
        &pool,
        id,
        "Updated",
        "updated-slug",
        "Updated Body",
        Some("New Desc"),
    )
    .await
    .expect("Failed to update draft");

    let fetched = DraftArticleService::fetch_by_id(&pool, id)
        .await
        .expect("Failed to fetch")
        .expect("Not found");

    assert_eq!(fetched.article.title, "Updated");
    assert_eq!(fetched.article.slug, "updated-slug");
    assert_eq!(fetched.article.body, "Updated Body");
    assert_eq!(fetched.article.description, Some("New Desc".to_string()));
}

#[sqlx::test]
async fn test_draft_service_delete(pool: PgPool) {
    let id = DraftArticleService::create(&pool, "To Delete", "to-delete", "Body", None)
        .await
        .expect("Failed to create draft");

    let before = DraftArticleService::fetch_by_id(&pool, id).await.unwrap();
    assert!(before.is_some());

    DraftArticleService::delete(&pool, id)
        .await
        .expect("Failed to delete");

    let after = DraftArticleService::fetch_by_id(&pool, id).await.unwrap();
    assert!(after.is_none());
}

#[sqlx::test]
async fn test_draft_service_publish(pool: PgPool) {
    let cat_id = create_test_category(&pool, "PublishCat", "publishcat").await;

    let draft_id = DraftArticleService::create(
        &pool,
        "Draft to Publish",
        "draft-to-publish",
        "Draft Body",
        Some("Draft Desc"),
    )
    .await
    .expect("Failed to create draft");

    link_draft_article_category(&pool, draft_id, cat_id).await;

    let published_id = DraftArticleService::publish(&pool, draft_id)
        .await
        .expect("Failed to publish");

    // 公開記事が作成されている
    let published = sqlx::query!(
        r#"SELECT slug, title, body, description FROM published_articles WHERE id = $1"#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch published article");

    assert_eq!(published.slug, "draft-to-publish");
    assert_eq!(published.title, "Draft to Publish");
    assert_eq!(published.body, "Draft Body");
    assert_eq!(published.description, Some("Draft Desc".to_string()));

    // カテゴリもコピーされている
    let category_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM published_article_categories WHERE article_id = $1"#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count categories");

    assert_eq!(category_count, 1);

    // 下書きは削除されている
    let draft_after = DraftArticleService::fetch_by_id(&pool, draft_id)
        .await
        .expect("Failed to fetch draft");
    assert!(draft_after.is_none());
}

#[sqlx::test]
async fn test_draft_service_publish_nonexistent_returns_not_found(pool: PgPool) {
    let nonexistent_id = Uuid::new_v4();
    let result = DraftArticleService::publish(&pool, nonexistent_id).await;
    assert!(matches!(result, Err(CmsError::NotFound)));
}

// ============================================================
// PublishedArticleService tests
// ============================================================

#[sqlx::test]
async fn test_published_service_fetch_all(pool: PgPool) {
    insert_published_article(
        &pool,
        "pub1",
        "Published 1",
        "Body",
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;
    insert_published_article(
        &pool,
        "pub2",
        "Published 2",
        "Body",
        parse_datetime("2025-01-02 10:00:00"),
    )
    .await;
    // 未来の記事
    insert_published_article(
        &pool,
        "future",
        "Future",
        "Body",
        parse_datetime("2099-01-01 10:00:00"),
    )
    .await;

    let service = PublishedArticleService::new(pool.clone());
    let result = service.fetch_all().await.expect("Failed to fetch all");

    // 未来の記事は含まれない
    assert_eq!(result.len(), 2);
}

#[sqlx::test]
async fn test_published_service_fetch_by_slug(pool: PgPool) {
    let article_id = insert_published_article(
        &pool,
        "unique-slug",
        "Unique Article",
        "Body",
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;

    let service = PublishedArticleService::new(pool.clone());
    let result = service
        .fetch_by_slug("unique-slug")
        .await
        .expect("Failed to fetch by slug");

    assert!(result.is_some());
    assert_eq!(result.unwrap().article.id, article_id);
}

#[sqlx::test]
async fn test_published_service_fetch_by_id(pool: PgPool) {
    let article_id = insert_published_article(
        &pool,
        "test-id-fetch",
        "Test ID Fetch",
        "Body",
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;

    let service = PublishedArticleService::new(pool.clone());
    let result = service
        .fetch_by_id(article_id)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_some());
    assert_eq!(result.unwrap().article.title, "Test ID Fetch");
}

// ============================================================
// AdminArticleService tests
// ============================================================

#[sqlx::test]
async fn test_admin_service_fetch_all(pool: PgPool) {
    insert_published_article(
        &pool,
        "admin-pub",
        "Admin Published",
        "Body",
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;
    insert_draft_article(&pool, "admin-draft", "Admin Draft", "Body").await;

    let result = AdminArticleService::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    assert_eq!(result.len(), 2);

    let has_published = result
        .iter()
        .any(|a| matches!(a, ArticleListItem::Published(_)));
    let has_draft = result
        .iter()
        .any(|a| matches!(a, ArticleListItem::Draft(_)));

    assert!(has_published);
    assert!(has_draft);
}
