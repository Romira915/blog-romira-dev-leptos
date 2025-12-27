use crate::error::CmsError;
use crate::models::{Category, PublishedArticle, PublishedArticleWithCategories};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 公開記事クエリサービス（SELECT操作）
pub struct PublishedArticleQuery;

impl PublishedArticleQuery {
    /// 公開済み記事一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(
        pool: &PgPool,
        now: NaiveDateTime,
    ) -> Result<Vec<PublishedArticleWithCategories>, CmsError> {
        let articles = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE published_at <= $1
            ORDER BY published_at DESC
            "#,
            now as _
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(articles.len());
        for article in articles {
            let categories = Self::fetch_categories(pool, article.id).await?;
            result.push(PublishedArticleWithCategories {
                article,
                categories,
            });
        }

        Ok(result)
    }

    /// 公開済み記事をIDで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_id(
        pool: &PgPool,
        article_id: Uuid,
        now: NaiveDateTime,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        let article = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE id = $1 AND published_at <= $2
            "#,
            article_id,
            now as _
        )
        .fetch_optional(pool)
        .await?;

        match article {
            Some(article) => {
                let categories = Self::fetch_categories(pool, article.id).await?;
                Ok(Some(PublishedArticleWithCategories {
                    article,
                    categories,
                }))
            }
            None => Ok(None),
        }
    }

    /// 公開済み記事をslugで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_slug(
        pool: &PgPool,
        slug: &str,
        now: NaiveDateTime,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        let article = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE slug = $1 AND published_at <= $2
            "#,
            slug,
            now as _
        )
        .fetch_optional(pool)
        .await?;

        match article {
            Some(article) => {
                let categories = Self::fetch_categories(pool, article.id).await?;
                Ok(Some(PublishedArticleWithCategories {
                    article,
                    categories,
                }))
            }
            None => Ok(None),
        }
    }

    /// 公開記事のカテゴリを取得
    #[instrument(skip(pool))]
    async fn fetch_categories(pool: &PgPool, article_id: Uuid) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN published_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_datetime(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
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

    async fn insert_published_article(
        pool: &PgPool,
        slug: &str,
        title: &str,
        body: &str,
        description: Option<&str>,
        published_at: NaiveDateTime,
    ) -> Uuid {
        sqlx::query_scalar!(
            r#"INSERT INTO published_articles (slug, title, body, description, published_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"#,
            slug,
            title,
            body,
            description,
            published_at as _
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert published article")
    }

    async fn link_published_article_category(pool: &PgPool, article_id: Uuid, category_id: Uuid) {
        sqlx::query!(
            "INSERT INTO published_article_categories (article_id, category_id) VALUES ($1, $2)",
            article_id,
            category_id
        )
        .execute(pool)
        .await
        .expect("Failed to link published article category");
    }

    #[sqlx::test]
    async fn test_fetch_all_returns_only_published_articles(pool: PgPool) {
        // 過去の公開日の記事
        let past_id = insert_published_article(
            &pool,
            "past-article",
            "Past Article",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;

        // 未来の公開日の記事
        insert_published_article(
            &pool,
            "future-article",
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

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].article.id, past_id);
    }

    #[sqlx::test]
    async fn test_fetch_all_includes_categories(pool: PgPool) {
        let cat_id = create_test_category(&pool, "TestCategory", "testcategory").await;
        let article_id = insert_published_article(
            &pool,
            "with-cat",
            "With Category",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;

        link_published_article_category(&pool, article_id, cat_id).await;

        let now = parse_datetime("2025-01-15 12:00:00");
        let result = PublishedArticleQuery::fetch_by_id(&pool, article_id, now)
            .await
            .expect("Failed to fetch")
            .expect("Article not found");

        assert_eq!(result.categories.len(), 1);
    }

    #[sqlx::test]
    async fn test_fetch_by_id_returns_article(pool: PgPool) {
        let article_id = insert_published_article(
            &pool,
            "test-slug",
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
        assert_eq!(article.article.slug, "test-slug");
        assert_eq!(article.article.description, Some("Test Desc".to_string()));
    }

    #[sqlx::test]
    async fn test_fetch_by_id_returns_none_for_future_article(pool: PgPool) {
        let article_id = insert_published_article(
            &pool,
            "future-slug",
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

    #[sqlx::test]
    async fn test_fetch_by_id_returns_none_for_nonexistent(pool: PgPool) {
        let nonexistent_id = Uuid::new_v4();
        let now = parse_datetime("2025-01-15 12:00:00");

        let result = PublishedArticleQuery::fetch_by_id(&pool, nonexistent_id, now)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_fetch_by_slug_returns_article(pool: PgPool) {
        let article_id = insert_published_article(
            &pool,
            "my-unique-slug",
            "Slug Article",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;

        let now = parse_datetime("2025-01-15 12:00:00");
        let result = PublishedArticleQuery::fetch_by_slug(&pool, "my-unique-slug", now)
            .await
            .expect("Failed to fetch by slug");

        assert!(result.is_some());
        assert_eq!(result.unwrap().article.id, article_id);
    }

    #[sqlx::test]
    async fn test_fetch_by_slug_returns_none_for_future_article(pool: PgPool) {
        insert_published_article(
            &pool,
            "future-slug",
            "Future",
            "Body",
            None,
            parse_datetime("2099-01-20 10:00:00"),
        )
        .await;

        let now = parse_datetime("2025-01-15 12:00:00");
        let result = PublishedArticleQuery::fetch_by_slug(&pool, "future-slug", now)
            .await
            .expect("Failed to fetch by slug");

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_fetch_by_slug_returns_none_for_nonexistent(pool: PgPool) {
        let now = parse_datetime("2025-01-15 12:00:00");
        let result = PublishedArticleQuery::fetch_by_slug(&pool, "nonexistent-slug", now)
            .await
            .expect("Failed to fetch by slug");

        assert!(result.is_none());
    }
}
