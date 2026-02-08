//noinspection NonAsciiCharacters
//! Service層の統合テスト
//! 複数のRepository/Query操作を組み合わせるワークフローをテスト

use blog_romira_dev_cms::error::CmsError;
use blog_romira_dev_cms::services::{DraftArticleService, PublishedArticleService};
use blog_romira_dev_cms::test_utils::*;
use blog_romira_dev_cms::{ArticleContent, PublishedArticleSlug, PublishedArticleTitle};
use sqlx::PgPool;
use uuid::Uuid;

//noinspection NonAsciiCharacters
/// 下書き記事を公開するワークフローのテスト
/// - 下書き作成 → カテゴリ紐付け → 公開
/// - 公開記事が作成され、カテゴリもコピーされ、下書きが削除されることを確認
#[sqlx::test]
async fn test_下書き公開で公開記事が作成されカテゴリがコピーされ下書きが削除されること(
    pool: PgPool,
) {
    let cat_id = create_test_category(&pool, "PublishCat", "publishcat").await;

    let service = DraftArticleService::new(pool.clone());

    let draft_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "Draft to Publish",
        slug: "draft-to-publish",
        body: "Draft Body",
        description: Some("Draft Desc"),
        cover_image_url: None,
    };
    service
        .save(draft_id, &content)
        .await
        .expect("Failed to create draft");

    link_draft_article_category(&pool, draft_id, cat_id).await;

    let published_id = service.publish(draft_id).await.expect("Failed to publish");

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
    let draft_after = service
        .fetch_by_id(draft_id)
        .await
        .expect("Failed to fetch draft");
    assert!(draft_after.is_none());
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_存在しない下書きを公開するとnotfoundエラーになること(
    pool: PgPool,
) {
    let service = DraftArticleService::new(pool);
    let nonexistent_id = Uuid::now_v7();
    let result = service.publish(nonexistent_id).await;
    assert!(matches!(result, Err(CmsError::NotFound)));
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_存在しない下書きをdeleteするとnotfoundエラーになること(
    pool: PgPool,
) {
    let service = DraftArticleService::new(pool);
    let nonexistent_id = Uuid::now_v7();
    let result = service.delete(nonexistent_id).await;
    assert!(matches!(result, Err(CmsError::NotFound)));
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_空スラッグの下書きを公開するとvalidationエラーになること(
    pool: PgPool,
) {
    let service = DraftArticleService::new(pool);

    let draft_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "Title",
        slug: "",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    service
        .save(draft_id, &content)
        .await
        .expect("Failed to create draft");

    let result = service.publish(draft_id).await;
    assert!(matches!(result, Err(CmsError::ValidationError(_))));

    // 下書きは削除されていない
    let draft = service
        .fetch_by_id(draft_id)
        .await
        .expect("Failed to fetch draft");
    assert!(draft.is_some());
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_重複スラッグの下書きを公開するとvalidationエラーになること(
    pool: PgPool,
) {
    let service = DraftArticleService::new(pool);

    // 先に同じスラッグで公開記事を作成
    let first_draft_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "First",
        slug: "duplicate-slug",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    service
        .save(first_draft_id, &content)
        .await
        .expect("Failed to create first draft");
    service
        .publish(first_draft_id)
        .await
        .expect("Failed to publish first draft");

    // 同じスラッグで下書きを作成して公開を試みる
    let second_draft_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "Second",
        slug: "duplicate-slug",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    service
        .save(second_draft_id, &content)
        .await
        .expect("Failed to create second draft");

    let result = service.publish(second_draft_id).await;
    assert!(matches!(result, Err(CmsError::ValidationError(_))));

    // 下書きは削除されていない
    let draft = service
        .fetch_by_id(second_draft_id)
        .await
        .expect("Failed to fetch draft");
    assert!(draft.is_some());
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_公開記事を同じスラッグで更新すると成功すること(pool: PgPool) {
    let draft_service = DraftArticleService::new(pool.clone());
    let published_service = PublishedArticleService::new(pool);

    // 公開記事を作成
    let draft_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "Original",
        slug: "same-slug",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    draft_service
        .save(draft_id, &content)
        .await
        .expect("Failed to create draft");
    let published_id = draft_service
        .publish(draft_id)
        .await
        .expect("Failed to publish");

    // 同じスラッグで更新
    let title = PublishedArticleTitle::new("Updated Title".to_string()).unwrap();
    let slug = PublishedArticleSlug::new("same-slug".to_string()).unwrap();
    let result = published_service
        .update(published_id, &title, &slug, "Updated Body", None, None)
        .await;

    assert!(result.is_ok());
}

//noinspection NonAsciiCharacters
#[sqlx::test]
async fn test_公開記事を他の記事と重複するスラッグで更新するとvalidationエラーになること(
    pool: PgPool,
) {
    let draft_service = DraftArticleService::new(pool.clone());
    let published_service = PublishedArticleService::new(pool);

    // 2つの公開記事を作成
    let draft1_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "First",
        slug: "first-slug",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    draft_service
        .save(draft1_id, &content)
        .await
        .expect("Failed to create first draft");
    let _first_published_id = draft_service
        .publish(draft1_id)
        .await
        .expect("Failed to publish first");

    let draft2_id = Uuid::now_v7();
    let content = ArticleContent {
        title: "Second",
        slug: "second-slug",
        body: "Body",
        description: None,
        cover_image_url: None,
    };
    draft_service
        .save(draft2_id, &content)
        .await
        .expect("Failed to create second draft");
    let second_published_id = draft_service
        .publish(draft2_id)
        .await
        .expect("Failed to publish second");

    // 2番目の記事を1番目と同じスラッグで更新しようとする
    let title = PublishedArticleTitle::new("Updated".to_string()).unwrap();
    let slug = PublishedArticleSlug::new("first-slug".to_string()).unwrap();
    let result = published_service
        .update(second_published_id, &title, &slug, "Body", None, None)
        .await;

    assert!(matches!(result, Err(CmsError::ValidationError(_))));
}
