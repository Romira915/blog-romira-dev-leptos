mod common;

use blog_romira_dev_cms::error::CmsError;
use blog_romira_dev_cms::models::ArticleListItem;
use blog_romira_dev_cms::services::{
    AdminArticleService, DraftArticleService, PublishedArticleService,
};
use chrono::NaiveDateTime;

fn parse_datetime(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
}

// ============================================================
// DraftArticleService tests
// ============================================================

#[tokio::test]
async fn test_draft_service_create_and_fetch() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    // 作成
    let id = DraftArticleService::create(
        &pool,
        "サービス経由タイトル",
        &format!("{}-service-slug", prefix),
        "サービス経由本文",
        Some("説明"),
    )
    .await
    .expect("Failed to create draft");

    // 取得
    let fetched = DraftArticleService::fetch_by_id(&pool, id)
        .await
        .expect("Failed to fetch by id")
        .expect("Article not found");

    assert_eq!(fetched.article.title, "サービス経由タイトル");
    assert_eq!(fetched.article.slug, format!("{}-service-slug", prefix));
    assert_eq!(fetched.article.body, "サービス経由本文");
    assert_eq!(fetched.article.description, Some("説明".to_string()));
}

#[tokio::test]
async fn test_draft_service_fetch_all() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let id1 = DraftArticleService::create(
        &pool,
        "Title 1",
        &format!("{}-slug1", prefix),
        "Body 1",
        None,
    )
    .await
    .expect("Failed to create draft 1");

    let id2 = DraftArticleService::create(
        &pool,
        "Title 2",
        &format!("{}-slug2", prefix),
        "Body 2",
        None,
    )
    .await
    .expect("Failed to create draft 2");

    let all = DraftArticleService::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_articles: Vec<_> = all
        .iter()
        .filter(|a| a.article.id == id1 || a.article.id == id2)
        .collect();

    assert_eq!(test_articles.len(), 2);
}

#[tokio::test]
async fn test_draft_service_update() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let id = DraftArticleService::create(
        &pool,
        "Original",
        &format!("{}-original-slug", prefix),
        "Original Body",
        None,
    )
    .await
    .expect("Failed to create draft");

    let updated_slug = format!("{}-updated-slug", prefix);
    DraftArticleService::update(
        &pool,
        id,
        "Updated",
        &updated_slug,
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
    assert_eq!(fetched.article.slug, updated_slug);
    assert_eq!(fetched.article.body, "Updated Body");
    assert_eq!(fetched.article.description, Some("New Desc".to_string()));
}

#[tokio::test]
async fn test_draft_service_delete() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let id = DraftArticleService::create(
        &pool,
        "To Delete",
        &format!("{}-to-delete", prefix),
        "Body",
        None,
    )
    .await
    .expect("Failed to create draft");

    // 削除前は存在する
    let before = DraftArticleService::fetch_by_id(&pool, id).await.unwrap();
    assert!(before.is_some());

    // 削除
    DraftArticleService::delete(&pool, id)
        .await
        .expect("Failed to delete");

    // 削除後は存在しない
    let after = DraftArticleService::fetch_by_id(&pool, id).await.unwrap();
    assert!(after.is_none());
}

#[tokio::test]
async fn test_draft_service_publish() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    // カテゴリを作成
    let cat_id = common::create_test_category(
        &pool,
        &format!("{}-PublishCat", prefix),
        &format!("{}-publishcat", prefix),
    )
    .await;

    // 下書きを作成
    let draft_id = DraftArticleService::create(
        &pool,
        "Draft to Publish",
        &format!("{}-draft-to-publish", prefix),
        "Draft Body",
        Some("Draft Desc"),
    )
    .await
    .expect("Failed to create draft");

    // カテゴリを関連付け
    common::link_draft_article_category(&pool, draft_id, cat_id).await;

    // 公開
    let published_id = DraftArticleService::publish(&pool, draft_id)
        .await
        .expect("Failed to publish");

    // 公開記事が作成されている
    let published = sqlx::query!(
        r#"
        SELECT slug, title, body, description
        FROM published_articles
        WHERE id = $1
        "#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch published article");

    assert_eq!(published.slug, format!("{}-draft-to-publish", prefix));
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

#[tokio::test]
async fn test_draft_service_publish_nonexistent_returns_not_found() {
    let pool = common::create_test_pool().await;

    let nonexistent_id = uuid::Uuid::new_v4();

    let result = DraftArticleService::publish(&pool, nonexistent_id).await;

    assert!(matches!(result, Err(CmsError::NotFound)));
}

// ============================================================
// PublishedArticleService tests
// ============================================================

#[tokio::test]
async fn test_published_service_fetch_all() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let id1 = common::insert_published_article_directly(
        &pool,
        &format!("{}-pub1", prefix),
        "Published 1",
        "Body",
        None,
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;

    let id2 = common::insert_published_article_directly(
        &pool,
        &format!("{}-pub2", prefix),
        "Published 2",
        "Body",
        None,
        parse_datetime("2025-01-02 10:00:00"),
    )
    .await;

    // 未来の記事
    let _future_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-future", prefix),
        "Future",
        "Body",
        None,
        parse_datetime("2099-01-01 10:00:00"),
    )
    .await;

    let service = PublishedArticleService::new(pool.clone());

    let result = service.fetch_all().await.expect("Failed to fetch all");

    // このテスト用のslugでフィルタして確認（未来の記事は含まれない）
    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| a.article.slug.starts_with(&prefix))
        .collect();

    assert_eq!(test_articles.len(), 2);
    assert!(test_articles.iter().any(|a| a.article.id == id1));
    assert!(test_articles.iter().any(|a| a.article.id == id2));
}

#[tokio::test]
async fn test_published_service_fetch_by_slug() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let slug = format!("{}-unique-slug", prefix);
    let article_id = common::insert_published_article_directly(
        &pool,
        &slug,
        "Unique Article",
        "Body",
        None,
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;

    let service = PublishedArticleService::new(pool.clone());

    let result = service
        .fetch_by_slug(&slug)
        .await
        .expect("Failed to fetch by slug");

    assert!(result.is_some());
    assert_eq!(result.unwrap().article.id, article_id);
}

#[tokio::test]
async fn test_published_service_fetch_by_id() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let article_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-test-id-fetch", prefix),
        "Test ID Fetch",
        "Body",
        None,
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

#[tokio::test]
async fn test_admin_service_fetch_all() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    // 公開記事と下書き記事を作成
    common::insert_published_article_directly(
        &pool,
        &format!("{}-admin-pub", prefix),
        "Admin Published",
        "Body",
        None,
        parse_datetime("2025-01-01 10:00:00"),
    )
    .await;

    common::insert_draft_article_directly(
        &pool,
        &format!("{}-admin-draft", prefix),
        "Admin Draft",
        "Body",
        None,
    )
    .await;

    let result = AdminArticleService::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| match a {
            ArticleListItem::Published(p) => p.article.slug.starts_with(&prefix),
            ArticleListItem::Draft(d) => d.article.slug.starts_with(&prefix),
        })
        .collect();

    assert_eq!(test_articles.len(), 2);
}
