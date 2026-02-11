use crate::error::CmsError;
use crate::models::{ArticleContent, DraftArticleWithCategories};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 公開記事リポジトリ（CUD操作）
pub struct PublishedArticleRepository;

impl PublishedArticleRepository {
    /// 下書きから公開記事を作成
    #[instrument(skip(pool, draft))]
    pub async fn create_from_draft(
        pool: &PgPool,
        draft: &DraftArticleWithCategories,
        now: NaiveDateTime,
    ) -> Result<Uuid, CmsError> {
        let published_id = sqlx::query_scalar!(
            r#"
            INSERT INTO published_articles (slug, title, body, description, cover_image_url, published_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6, $6)
            RETURNING id
            "#,
            &draft.article.slug,
            &draft.article.title,
            &draft.article.body,
            draft.article.description.as_deref(),
            draft.article.cover_image_url.as_deref(),
            now as _
        )
        .fetch_one(pool)
        .await?;

        // カテゴリをコピー
        for category in &draft.categories {
            sqlx::query!(
                "INSERT INTO published_article_categories (article_id, category_id) VALUES ($1, $2)",
                published_id,
                category.id
            )
            .execute(pool)
            .await?;
        }

        Ok(published_id)
    }

    /// 公開記事を削除
    #[instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, article_id: Uuid) -> Result<(), CmsError> {
        let rows = sqlx::query!("DELETE FROM published_articles WHERE id = $1", article_id)
            .execute(pool)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(CmsError::NotFound);
        }

        Ok(())
    }

    /// 公開記事を更新
    #[instrument(skip(pool))]
    pub async fn update(
        pool: &PgPool,
        article_id: Uuid,
        content: &ArticleContent<'_>,
        now: NaiveDateTime,
    ) -> Result<(), CmsError> {
        let rows = sqlx::query!(
            r#"
            UPDATE published_articles
            SET title = $1, slug = $2, body = $3, description = $4, cover_image_url = $5, updated_at = $6
            WHERE id = $7
            "#,
            content.title,
            content.slug,
            content.body,
            content.description,
            content.cover_image_url,
            now as _,
            article_id
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(CmsError::NotFound);
        }

        Ok(())
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArticleContent, Category, DraftArticle};
    use crate::test_utils::*;

    #[sqlx::test]
    async fn test_カテゴリなし下書きから公開記事が作成されること(
        pool: PgPool,
    ) {
        let draft_id =
            insert_draft_article(&pool, "draft-slug", "下書きタイトル", "下書き本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "draft-slug".to_string(),
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

        assert_eq!(published.slug, "draft-slug");
        assert_eq!(published.title, "下書きタイトル");
        assert_eq!(published.body, "下書き本文");
        assert_eq!(published.description, Some("下書き説明".to_string()));
    }

    #[sqlx::test]
    async fn test_カテゴリ付き下書きから公開記事とカテゴリが作成されること(
        pool: PgPool,
    ) {
        let cat1_id = create_test_category(&pool, "Cat1", "cat1").await;
        let cat2_id = create_test_category(&pool, "Cat2", "cat2").await;
        let draft_id =
            insert_draft_article(&pool, "draft-with-cat", "カテゴリ付き", "本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "draft-with-cat".to_string(),
                title: "カテゴリ付き".to_string(),
                body: "本文".to_string(),
                description: None,
                cover_image_url: None,
                created_at: utc_now(),
                updated_at: utc_now(),
            },
            categories: vec![
                Category {
                    id: cat1_id,
                    name: "Cat1".to_string(),
                    slug: "cat1".to_string(),
                },
                Category {
                    id: cat2_id,
                    name: "Cat2".to_string(),
                    slug: "cat2".to_string(),
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

    #[sqlx::test]
    async fn test_公開記事作成時に公開日時が設定されること(pool: PgPool) {
        let draft_id = insert_draft_article(&pool, "time-test", "時刻テスト", "本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "time-test".to_string(),
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
        let published_id =
            PublishedArticleRepository::create_from_draft(&pool, &draft, publish_time)
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

    #[sqlx::test]
    async fn test_updateで公開記事が更新されること(pool: PgPool) {
        let draft_id =
            insert_draft_article(&pool, "original-slug", "元のタイトル", "元の本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "original-slug".to_string(),
                title: "元のタイトル".to_string(),
                body: "元の本文".to_string(),
                description: Some("元の説明".to_string()),
                cover_image_url: None,
                created_at: utc_now(),
                updated_at: utc_now(),
            },
            categories: vec![],
        };

        let publish_time = utc_now();
        let published_id =
            PublishedArticleRepository::create_from_draft(&pool, &draft, publish_time)
                .await
                .expect("Failed to create published article");

        let update_time = utc_now();
        let content = ArticleContent {
            title: "更新後のタイトル",
            slug: "updated-slug",
            body: "更新後の本文",
            description: Some("更新後の説明"),
            cover_image_url: None,
        };
        PublishedArticleRepository::update(&pool, published_id, &content, update_time)
            .await
            .expect("Failed to update published article");

        let updated = sqlx::query!(
            r#"SELECT title, slug, body, description FROM published_articles WHERE id = $1"#,
            published_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated article");

        assert_eq!(updated.title, "更新後のタイトル");
        assert_eq!(updated.slug, "updated-slug");
        assert_eq!(updated.body, "更新後の本文");
        assert_eq!(updated.description, Some("更新後の説明".to_string()));
    }

    #[sqlx::test]
    async fn test_存在しない公開記事をupdateするとnotfoundエラーになること(
        pool: PgPool,
    ) {
        let nonexistent_id = Uuid::now_v7();
        let content = ArticleContent {
            title: "タイトル",
            slug: "slug",
            body: "本文",
            description: None,
            cover_image_url: None,
        };
        let result =
            PublishedArticleRepository::update(&pool, nonexistent_id, &content, utc_now()).await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }

    #[sqlx::test]
    async fn test_updateでcover_image_urlが更新されること(pool: PgPool) {
        let draft_id =
            insert_draft_article(&pool, "cover-slug", "カバーテスト", "本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "cover-slug".to_string(),
                title: "カバーテスト".to_string(),
                body: "本文".to_string(),
                description: None,
                cover_image_url: None,
                created_at: utc_now(),
                updated_at: utc_now(),
            },
            categories: vec![],
        };

        let publish_time = utc_now();
        let published_id =
            PublishedArticleRepository::create_from_draft(&pool, &draft, publish_time)
                .await
                .expect("Failed to create published article");

        // cover_image_url を設定して更新
        let content = ArticleContent {
            title: "カバーテスト",
            slug: "cover-slug",
            body: "本文",
            description: None,
            cover_image_url: Some("https://example.com/cover.jpg"),
        };
        PublishedArticleRepository::update(&pool, published_id, &content, utc_now())
            .await
            .expect("Failed to update");

        let updated = sqlx::query!(
            r#"SELECT cover_image_url FROM published_articles WHERE id = $1"#,
            published_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(
            updated.cover_image_url.as_deref(),
            Some("https://example.com/cover.jpg")
        );

        // cover_image_url を None に戻す
        let content = ArticleContent {
            title: "カバーテスト",
            slug: "cover-slug",
            body: "本文",
            description: None,
            cover_image_url: None,
        };
        PublishedArticleRepository::update(&pool, published_id, &content, utc_now())
            .await
            .expect("Failed to update");

        let updated = sqlx::query!(
            r#"SELECT cover_image_url FROM published_articles WHERE id = $1"#,
            published_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(updated.cover_image_url, None);
    }

    #[sqlx::test]
    async fn test_deleteで公開記事が削除されること(pool: PgPool) {
        let draft_id = insert_draft_article(&pool, "delete-test", "削除対象", "本文", None).await;

        let draft = DraftArticleWithCategories {
            article: DraftArticle {
                id: draft_id,
                slug: "delete-test".to_string(),
                title: "削除対象".to_string(),
                body: "本文".to_string(),
                description: None,
                cover_image_url: None,
                created_at: utc_now(),
                updated_at: utc_now(),
            },
            categories: vec![],
        };

        let published_id = PublishedArticleRepository::create_from_draft(&pool, &draft, utc_now())
            .await
            .expect("Failed to create published article");

        let exists_before = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM published_articles WHERE id = $1) as "exists!""#,
            published_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check existence");
        assert!(exists_before);

        PublishedArticleRepository::delete(&pool, published_id)
            .await
            .expect("Failed to delete published article");

        let exists_after = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM published_articles WHERE id = $1) as "exists!""#,
            published_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check existence");
        assert!(!exists_after);
    }

    #[sqlx::test]
    async fn test_存在しない公開記事をdeleteするとnotfoundエラーになること(
        pool: PgPool,
    ) {
        let nonexistent_id = Uuid::now_v7();
        let result = PublishedArticleRepository::delete(&pool, nonexistent_id).await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }
}
