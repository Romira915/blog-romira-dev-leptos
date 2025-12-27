mod common;

use blog_romira_dev_cms::error::CmsError;
use blog_romira_dev_cms::repositories::DraftArticleRepository;
use chrono::{NaiveDateTime, Utc};

fn utc_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

#[tokio::test]
async fn test_create_draft_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let now = utc_now();
    let title = "テスト記事タイトル";
    let slug = format!("{}-test-article-slug", prefix);
    let body = "これはテスト記事の本文です。";
    let description = Some("テスト記事の説明");

    let id = DraftArticleRepository::create(&pool, title, &slug, body, description, now)
        .await
        .expect("Failed to create draft article");

    assert!(!id.is_nil());

    let article = sqlx::query!(
        r#"SELECT title, slug, body, description FROM draft_articles WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch created article");

    assert_eq!(article.title, title);
    assert_eq!(article.slug, slug);
    assert_eq!(article.body, body);
    assert_eq!(article.description.as_deref(), description);
}

#[tokio::test]
async fn test_create_draft_article_without_description() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let now = utc_now();
    let slug = format!("{}-no-desc", prefix);

    let id = DraftArticleRepository::create(&pool, "説明なし記事", &slug, "本文のみ", None, now)
        .await
        .expect("Failed to create draft article without description");

    let article = sqlx::query!(
        r#"SELECT description FROM draft_articles WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch created article");

    assert!(article.description.is_none());
}

#[tokio::test]
async fn test_update_draft_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let now = utc_now();
    let id = DraftArticleRepository::create(
        &pool,
        "元のタイトル",
        &format!("{}-original", prefix),
        "元の本文",
        Some("元の説明"),
        now,
    )
    .await
    .expect("Failed to create draft article");

    let updated_slug = format!("{}-updated", prefix);
    DraftArticleRepository::update(
        &pool,
        id,
        "更新後のタイトル",
        &updated_slug,
        "更新後の本文",
        Some("更新後の説明"),
        utc_now(),
    )
    .await
    .expect("Failed to update draft article");

    let article = sqlx::query!(
        r#"SELECT title, slug, body, description FROM draft_articles WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated article");

    assert_eq!(article.title, "更新後のタイトル");
    assert_eq!(article.slug, updated_slug);
    assert_eq!(article.body, "更新後の本文");
    assert_eq!(article.description, Some("更新後の説明".to_string()));
}

#[tokio::test]
async fn test_update_nonexistent_article_returns_not_found() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let nonexistent_id = uuid::Uuid::new_v4();

    let result = DraftArticleRepository::update(
        &pool,
        nonexistent_id,
        "タイトル",
        &format!("{}-nonexistent", prefix),
        "本文",
        None,
        utc_now(),
    )
    .await;

    assert!(matches!(result, Err(CmsError::NotFound)));
}

#[tokio::test]
async fn test_delete_draft_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let now = utc_now();
    let id = DraftArticleRepository::create(
        &pool,
        "削除対象",
        &format!("{}-to-delete", prefix),
        "本文",
        None,
        now,
    )
    .await
    .expect("Failed to create draft article");

    let exists_before = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM draft_articles WHERE id = $1) as "exists!""#,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check existence");
    assert!(exists_before);

    DraftArticleRepository::delete(&pool, id)
        .await
        .expect("Failed to delete draft article");

    let exists_after = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM draft_articles WHERE id = $1) as "exists!""#,
        id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to check existence");
    assert!(!exists_after);
}

#[tokio::test]
async fn test_delete_nonexistent_article_returns_not_found() {
    let pool = common::create_test_pool().await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let result = DraftArticleRepository::delete(&pool, nonexistent_id).await;

    assert!(matches!(result, Err(CmsError::NotFound)));
}
