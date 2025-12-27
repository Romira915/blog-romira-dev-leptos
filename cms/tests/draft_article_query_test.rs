mod common;

use blog_romira_dev_cms::queries::DraftArticleQuery;

#[tokio::test]
async fn test_fetch_all_includes_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let cat1_id = common::create_test_category(
        &pool,
        &format!("{}-Category1", prefix),
        &format!("{}-category1", prefix),
    )
    .await;
    let cat2_id = common::create_test_category(
        &pool,
        &format!("{}-Category2", prefix),
        &format!("{}-category2", prefix),
    )
    .await;

    let article_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-slug", prefix),
        "Title",
        "Body",
        None,
    )
    .await;

    common::link_draft_article_category(&pool, article_id, cat1_id).await;
    common::link_draft_article_category(&pool, article_id, cat2_id).await;

    let fetched = DraftArticleQuery::fetch_by_id(&pool, article_id)
        .await
        .expect("Failed to fetch")
        .expect("Article not found");

    assert_eq!(fetched.categories.len(), 2);
}

#[tokio::test]
async fn test_fetch_by_id_returns_article_with_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let cat_id = common::create_test_category(
        &pool,
        &format!("{}-TestCat", prefix),
        &format!("{}-testcat", prefix),
    )
    .await;

    let article_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-test-slug", prefix),
        "Test Title",
        "Test Body",
        Some("Test Description"),
    )
    .await;

    common::link_draft_article_category(&pool, article_id, cat_id).await;

    let result = DraftArticleQuery::fetch_by_id(&pool, article_id)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_some());
    let article_with_cats = result.unwrap();

    assert_eq!(article_with_cats.article.id, article_id);
    assert_eq!(
        article_with_cats.article.slug,
        format!("{}-test-slug", prefix)
    );
    assert_eq!(article_with_cats.article.title, "Test Title");
    assert_eq!(article_with_cats.categories.len(), 1);
}

#[tokio::test]
async fn test_fetch_by_id_returns_none_for_nonexistent() {
    let pool = common::create_test_pool().await;

    let nonexistent_id = uuid::Uuid::new_v4();

    let result = DraftArticleQuery::fetch_by_id(&pool, nonexistent_id)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_fetch_by_id_returns_article_without_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let article_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-no-cat-slug", prefix),
        "No Cat Title",
        "Body",
        None,
    )
    .await;

    let result = DraftArticleQuery::fetch_by_id(&pool, article_id)
        .await
        .expect("Failed to fetch by id");

    assert!(result.is_some());
    let article_with_cats = result.unwrap();

    assert_eq!(article_with_cats.article.id, article_id);
    assert!(article_with_cats.categories.is_empty());
}
