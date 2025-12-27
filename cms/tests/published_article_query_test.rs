mod common;

use blog_romira_dev_cms::queries::PublishedArticleQuery;
use chrono::NaiveDateTime;

fn parse_datetime(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[tokio::test]
async fn test_fetch_all_returns_only_published_articles() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    // 過去の公開日の記事
    let past_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-past-article", prefix),
        "Past Article",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    // 未来の公開日の記事
    let _future_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-future-article", prefix),
        "Future Article",
        "Body",
        None,
        parse_datetime("2099-01-20 10:00:00"),
    )
    .await;

    let now = parse_datetime("2025-01-15 12:00:00");

    let result = PublishedArticleQuery::fetch_all(&pool, now)
        .await
        .expect("Failed to fetch all");

    // このテスト用のslugでフィルタして確認
    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| a.article.slug.starts_with(&prefix))
        .collect();

    assert_eq!(test_articles.len(), 1);
    assert_eq!(test_articles[0].article.id, past_id);
}

#[tokio::test]
async fn test_fetch_all_includes_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let cat_id = common::create_test_category(
        &pool,
        &format!("{}-TestCategory", prefix),
        &format!("{}-testcategory", prefix),
    )
    .await;

    let article_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-with-cat", prefix),
        "With Category",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    common::link_published_article_category(&pool, article_id, cat_id).await;

    let now = parse_datetime("2025-01-15 12:00:00");
    let result = PublishedArticleQuery::fetch_by_id(&pool, article_id, now)
        .await
        .expect("Failed to fetch")
        .expect("Article not found");

    assert_eq!(result.categories.len(), 1);
}

#[tokio::test]
async fn test_fetch_by_id_returns_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let article_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-test-slug", prefix),
        "Test Title",
        "Test Body",
        Some("Test Desc"),
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    let now = parse_datetime("2025-01-15 12:00:00");
    let result = PublishedArticleQuery::fetch_by_id(&pool, article_id, now)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_some());
    let article = result.unwrap();
    assert_eq!(article.article.id, article_id);
    assert_eq!(article.article.slug, format!("{}-test-slug", prefix));
    assert_eq!(article.article.description, Some("Test Desc".to_string()));
}

#[tokio::test]
async fn test_fetch_by_id_returns_none_for_future_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let article_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-future-slug", prefix),
        "Future Title",
        "Body",
        None,
        parse_datetime("2099-01-20 10:00:00"),
    )
    .await;

    let now = parse_datetime("2025-01-15 12:00:00");
    let result = PublishedArticleQuery::fetch_by_id(&pool, article_id, now)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_fetch_by_id_returns_none_for_nonexistent() {
    let pool = common::create_test_pool().await;

    let nonexistent_id = uuid::Uuid::new_v4();
    let now = parse_datetime("2025-01-15 12:00:00");

    let result = PublishedArticleQuery::fetch_by_id(&pool, nonexistent_id, now)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_fetch_by_slug_returns_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let slug = format!("{}-my-unique-slug", prefix);
    let article_id = common::insert_published_article_directly(
        &pool,
        &slug,
        "Slug Article",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    let now = parse_datetime("2025-01-15 12:00:00");
    let result = PublishedArticleQuery::fetch_by_slug(&pool, &slug, now)
        .await
        .expect("Failed to fetch by slug");

    assert!(result.is_some());
    let article = result.unwrap();
    assert_eq!(article.article.id, article_id);
}

#[tokio::test]
async fn test_fetch_by_slug_returns_none_for_future_article() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let slug = format!("{}-future-slug", prefix);
    common::insert_published_article_directly(
        &pool,
        &slug,
        "Future",
        "Body",
        None,
        parse_datetime("2099-01-20 10:00:00"),
    )
    .await;

    let now = parse_datetime("2025-01-15 12:00:00");
    let result = PublishedArticleQuery::fetch_by_slug(&pool, &slug, now)
        .await
        .expect("Failed to fetch by slug");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_fetch_by_slug_returns_none_for_nonexistent() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let now = parse_datetime("2025-01-15 12:00:00");
    let result =
        PublishedArticleQuery::fetch_by_slug(&pool, &format!("{}-nonexistent-slug", prefix), now)
            .await
            .expect("Failed to fetch by slug");

    assert!(result.is_none());
}
