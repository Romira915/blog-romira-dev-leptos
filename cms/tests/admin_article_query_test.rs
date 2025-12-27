mod common;

use blog_romira_dev_cms::models::ArticleListItem;
use blog_romira_dev_cms::queries::AdminArticleQuery;
use chrono::NaiveDateTime;

fn parse_datetime(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
}

#[tokio::test]
async fn test_fetch_all_returns_only_published_articles() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let published_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-pub-slug", prefix),
        "Published",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| match a {
            ArticleListItem::Published(p) => p.article.slug.starts_with(&prefix),
            ArticleListItem::Draft(d) => d.article.slug.starts_with(&prefix),
        })
        .collect();

    assert_eq!(test_articles.len(), 1);
    assert!(matches!(test_articles[0], ArticleListItem::Published(_)));
    assert_eq!(test_articles[0].id(), published_id);
}

#[tokio::test]
async fn test_fetch_all_returns_only_draft_articles() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft-slug", prefix),
        "Draft",
        "Body",
        None,
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| match a {
            ArticleListItem::Published(p) => p.article.slug.starts_with(&prefix),
            ArticleListItem::Draft(d) => d.article.slug.starts_with(&prefix),
        })
        .collect();

    assert_eq!(test_articles.len(), 1);
    assert!(matches!(test_articles[0], ArticleListItem::Draft(_)));
    assert_eq!(test_articles[0].id(), draft_id);
}

#[tokio::test]
async fn test_fetch_all_returns_both_published_and_draft() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    common::insert_published_article_directly(
        &pool,
        &format!("{}-pub-slug", prefix),
        "Published",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft-slug", prefix),
        "Draft",
        "Body",
        None,
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
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

    let has_published = test_articles
        .iter()
        .any(|a| matches!(a, ArticleListItem::Published(_)));
    let has_draft = test_articles
        .iter()
        .any(|a| matches!(a, ArticleListItem::Draft(_)));

    assert!(has_published);
    assert!(has_draft);
}

#[tokio::test]
async fn test_fetch_all_includes_future_published_articles() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let future_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-future-slug", prefix),
        "Future Article",
        "Body",
        None,
        parse_datetime("2099-01-01 10:00:00"),
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_articles: Vec<_> = result
        .iter()
        .filter(|a| match a {
            ArticleListItem::Published(p) => p.article.slug.starts_with(&prefix),
            ArticleListItem::Draft(d) => d.article.slug.starts_with(&prefix),
        })
        .collect();

    assert_eq!(test_articles.len(), 1);
    assert_eq!(test_articles[0].id(), future_id);
}

#[tokio::test]
async fn test_fetch_all_includes_categories_for_published() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let cat_id = common::create_test_category(
        &pool,
        &format!("{}-PubCat", prefix),
        &format!("{}-pubcat", prefix),
    )
    .await;

    let article_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-cat-pub", prefix),
        "Categorized Published",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    common::link_published_article_category(&pool, article_id, cat_id).await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_article = result
        .iter()
        .find(|a| a.id() == article_id)
        .expect("Article not found");

    match test_article {
        ArticleListItem::Published(p) => {
            assert_eq!(p.categories.len(), 1);
        }
        _ => panic!("Expected Published article"),
    }
}

#[tokio::test]
async fn test_fetch_all_includes_categories_for_draft() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let cat_id = common::create_test_category(
        &pool,
        &format!("{}-DraftCat", prefix),
        &format!("{}-draftcat", prefix),
    )
    .await;

    let article_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-cat-draft", prefix),
        "Categorized Draft",
        "Body",
        None,
    )
    .await;

    common::link_draft_article_category(&pool, article_id, cat_id).await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let test_article = result
        .iter()
        .find(|a| a.id() == article_id)
        .expect("Article not found");

    match test_article {
        ArticleListItem::Draft(d) => {
            assert_eq!(d.categories.len(), 1);
        }
        _ => panic!("Expected Draft article"),
    }
}

#[tokio::test]
async fn test_article_list_item_is_draft() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let published_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-pub", prefix),
        "Published",
        "Body",
        None,
        parse_datetime("2020-01-01 10:00:00"),
    )
    .await;

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft", prefix),
        "Draft",
        "Body",
        None,
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let published = result.iter().find(|a| a.id() == published_id).unwrap();
    let draft = result.iter().find(|a| a.id() == draft_id).unwrap();

    assert!(!published.is_draft());
    assert!(draft.is_draft());
}

#[tokio::test]
async fn test_article_list_item_published_at() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let publish_time = parse_datetime("2025-06-15 14:30:00");
    let published_id = common::insert_published_article_directly(
        &pool,
        &format!("{}-pub", prefix),
        "Published",
        "Body",
        None,
        publish_time,
    )
    .await;

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft", prefix),
        "Draft",
        "Body",
        None,
    )
    .await;

    let result = AdminArticleQuery::fetch_all(&pool)
        .await
        .expect("Failed to fetch all");

    let published = result.iter().find(|a| a.id() == published_id).unwrap();
    let draft = result.iter().find(|a| a.id() == draft_id).unwrap();

    assert_eq!(published.published_at(), Some(publish_time));
    assert_eq!(draft.published_at(), None);
}
