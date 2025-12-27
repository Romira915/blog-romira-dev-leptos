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
