mod common;

use blog_romira_dev_cms::models::{Category, DraftArticle, DraftArticleWithCategories};
use blog_romira_dev_cms::repositories::PublishedArticleRepository;
use chrono::{NaiveDateTime, Utc};

fn utc_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

#[tokio::test]
async fn test_create_from_draft_without_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft-slug", prefix),
        "下書きタイトル",
        "下書き本文",
        Some("下書き説明"),
    )
    .await;

    let draft = DraftArticleWithCategories {
        article: DraftArticle {
            id: draft_id,
            slug: format!("{}-draft-slug", prefix),
            title: "下書きタイトル".to_string(),
            body: "下書き本文".to_string(),
            description: Some("下書き説明".to_string()),
            cover_image_url: None,
            created_at: utc_now(),
            updated_at: utc_now(),
        },
        categories: vec![],
    };

    let now = utc_now();
    let published_id = PublishedArticleRepository::create_from_draft(&pool, &draft, now)
        .await
        .expect("Failed to create published article from draft");

    let published = sqlx::query!(
        r#"SELECT slug, title, body, description FROM published_articles WHERE id = $1"#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch published article");

    assert_eq!(published.slug, format!("{}-draft-slug", prefix));
    assert_eq!(published.title, "下書きタイトル");
    assert_eq!(published.body, "下書き本文");
    assert_eq!(published.description, Some("下書き説明".to_string()));
}

#[tokio::test]
async fn test_create_from_draft_with_categories() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let category1_id = common::create_test_category(
        &pool,
        &format!("{}-Cat1", prefix),
        &format!("{}-cat1", prefix),
    )
    .await;
    let category2_id = common::create_test_category(
        &pool,
        &format!("{}-Cat2", prefix),
        &format!("{}-cat2", prefix),
    )
    .await;

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-draft-with-cat", prefix),
        "カテゴリ付き下書き",
        "本文",
        None,
    )
    .await;

    common::link_draft_article_category(&pool, draft_id, category1_id).await;
    common::link_draft_article_category(&pool, draft_id, category2_id).await;

    let draft = DraftArticleWithCategories {
        article: DraftArticle {
            id: draft_id,
            slug: format!("{}-draft-with-cat", prefix),
            title: "カテゴリ付き下書き".to_string(),
            body: "本文".to_string(),
            description: None,
            cover_image_url: None,
            created_at: utc_now(),
            updated_at: utc_now(),
        },
        categories: vec![
            Category {
                id: category1_id,
                name: format!("{}-Cat1", prefix),
                slug: format!("{}-cat1", prefix),
            },
            Category {
                id: category2_id,
                name: format!("{}-Cat2", prefix),
                slug: format!("{}-cat2", prefix),
            },
        ],
    };

    let now = utc_now();
    let published_id = PublishedArticleRepository::create_from_draft(&pool, &draft, now)
        .await
        .expect("Failed to create published article from draft");

    let category_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM published_article_categories WHERE article_id = $1"#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count categories");

    assert_eq!(category_count, 2);
}

#[tokio::test]
async fn test_create_from_draft_sets_published_at() {
    let pool = common::create_test_pool().await;
    let prefix = common::unique_prefix();

    let draft_id = common::insert_draft_article_directly(
        &pool,
        &format!("{}-time-test", prefix),
        "時刻テスト",
        "本文",
        None,
    )
    .await;

    let draft = DraftArticleWithCategories {
        article: DraftArticle {
            id: draft_id,
            slug: format!("{}-time-test", prefix),
            title: "時刻テスト".to_string(),
            body: "本文".to_string(),
            description: None,
            cover_image_url: None,
            created_at: utc_now(),
            updated_at: utc_now(),
        },
        categories: vec![],
    };

    let publish_time =
        NaiveDateTime::parse_from_str("2025-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();

    let published_id = PublishedArticleRepository::create_from_draft(&pool, &draft, publish_time)
        .await
        .expect("Failed to create published article");

    let published = sqlx::query!(
        r#"
        SELECT published_at as "published_at: NaiveDateTime",
               created_at as "created_at: NaiveDateTime",
               updated_at as "updated_at: NaiveDateTime"
        FROM published_articles WHERE id = $1
        "#,
        published_id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch published article");

    assert_eq!(published.published_at, publish_time);
    assert_eq!(published.created_at, publish_time);
    assert_eq!(published.updated_at, publish_time);
}
