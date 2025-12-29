use crate::error::CmsError;
use crate::models::DraftArticleWithCategories;
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

    /// 公開記事を更新
    #[instrument(skip(pool))]
    pub async fn update(
        pool: &PgPool,
        article_id: Uuid,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
        now: NaiveDateTime,
    ) -> Result<(), CmsError> {
        let rows = sqlx::query!(
            r#"
            UPDATE published_articles
            SET title = $1, slug = $2, body = $3, description = $4, updated_at = $5
            WHERE id = $6
            "#,
            title,
            slug,
            body,
            description,
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
    use crate::models::{Category, DraftArticle};
    use chrono::Utc;

    fn utc_now() -> NaiveDateTime {
        Utc::now().naive_utc()
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

    async fn insert_draft_article(pool: &PgPool, slug: &str, title: &str, body: &str) -> Uuid {
        let now = utc_now();
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

    #[sqlx::test]
    async fn test_カテゴリなし下書きから公開記事が作成されること(
        pool: PgPool,
    ) {
        let draft_id =
            insert_draft_article(&pool, "draft-slug", "下書きタイトル", "下書き本文").await;

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
        let draft_id = insert_draft_article(&pool, "draft-with-cat", "カテゴリ付き", "本文").await;

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
        let draft_id = insert_draft_article(&pool, "time-test", "時刻テスト", "本文").await;

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
            insert_draft_article(&pool, "original-slug", "元のタイトル", "元の本文").await;

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
        PublishedArticleRepository::update(
            &pool,
            published_id,
            "更新後のタイトル",
            "updated-slug",
            "更新後の本文",
            Some("更新後の説明"),
            update_time,
        )
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
        let result = PublishedArticleRepository::update(
            &pool,
            nonexistent_id,
            "タイトル",
            "slug",
            "本文",
            None,
            utc_now(),
        )
        .await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }
}
