use crate::error::CmsError;
use crate::models::{Article, ArticleWithCategories, Category};
use chrono::Utc;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ArticleService {
    pool: PgPool,
}

impl ArticleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 公開済み記事を取得（draft=false かつ published_at <= now）
    #[instrument(skip(self))]
    pub async fn fetch_published_articles(
        &self,
    ) -> Result<Vec<ArticleWithCategories>, CmsError> {
        let articles = sqlx::query_as::<_, Article>(
            r#"
            SELECT id, slug, title, body, description, cover_image_url, draft, published_at, created_at, updated_at
            FROM articles
            WHERE draft = false AND published_at <= $1
            ORDER BY published_at DESC
            "#,
        )
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(articles.len());
        for article in articles {
            let categories = self.fetch_categories_for_article(article.id).await?;
            result.push(ArticleWithCategories { article, categories });
        }

        Ok(result)
    }

    /// 記事をIDで取得（公開済みのみ）
    #[instrument(skip(self))]
    pub async fn fetch_published_article(
        &self,
        article_id: Uuid,
    ) -> Result<Option<ArticleWithCategories>, CmsError> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            SELECT id, slug, title, body, description, cover_image_url, draft, published_at, created_at, updated_at
            FROM articles
            WHERE id = $1 AND draft = false AND published_at <= $2
            "#,
        )
        .bind(article_id)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        match article {
            Some(article) => {
                let categories = self.fetch_categories_for_article(article.id).await?;
                Ok(Some(ArticleWithCategories { article, categories }))
            }
            None => Ok(None),
        }
    }

    /// 記事をIDで取得（プレビュー用、draft含む）
    #[instrument(skip(self))]
    pub async fn fetch_preview_article(
        &self,
        article_id: Uuid,
    ) -> Result<Option<ArticleWithCategories>, CmsError> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            SELECT id, slug, title, body, description, cover_image_url, draft, published_at, created_at, updated_at
            FROM articles
            WHERE id = $1
            "#,
        )
        .bind(article_id)
        .fetch_optional(&self.pool)
        .await?;

        match article {
            Some(article) => {
                let categories = self.fetch_categories_for_article(article.id).await?;
                Ok(Some(ArticleWithCategories { article, categories }))
            }
            None => Ok(None),
        }
    }

    /// 記事に紐づくカテゴリを取得
    #[instrument(skip(self))]
    async fn fetch_categories_for_article(
        &self,
        article_id: Uuid,
    ) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as::<_, Category>(
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
        )
        .bind(article_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }
}
