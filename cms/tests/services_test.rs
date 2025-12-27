//! Service層の統合テスト
//! 複数のRepository/Query操作を組み合わせるワークフローをテスト

use blog_romira_dev_cms::error::CmsError;
use blog_romira_dev_cms::services::DraftArticleService;
use sqlx::PgPool;
use uuid::Uuid;

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

/// 下書き記事を公開するワークフローのテスト
/// - 下書き作成 → カテゴリ紐付け → 公開
/// - 公開記事が作成され、カテゴリもコピーされ、下書きが削除されることを確認
#[sqlx::test]
async fn test_下書き公開で公開記事が作成されカテゴリがコピーされ下書きが削除されること(pool: PgPool) {
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
async fn test_存在しない下書きを公開するとnotfoundエラーになること(pool: PgPool) {
    let nonexistent_id = Uuid::new_v4();
    let result = DraftArticleService::publish(&pool, nonexistent_id).await;
    assert!(matches!(result, Err(CmsError::NotFound)));
}
