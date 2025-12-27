use crate::error::CmsError;
use crate::models::{Category, DraftArticle, DraftArticleWithCategories};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 下書き記事クエリサービス（SELECT操作）
pub struct DraftArticleQuery;

impl DraftArticleQuery {
    /// 下書き記事一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<DraftArticleWithCategories>, CmsError> {
        let articles = sqlx::query_as!(
            DraftArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   created_at as "created_at: _", updated_at as "updated_at: _"
            FROM draft_articles
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(articles.len());
        for article in articles {
            let categories = Self::fetch_categories(pool, article.id).await?;
            result.push(DraftArticleWithCategories {
                article,
                categories,
            });
        }

        Ok(result)
    }

    /// 下書き記事をIDで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_id(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Option<DraftArticleWithCategories>, CmsError> {
        let article = sqlx::query_as!(
            DraftArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   created_at as "created_at: _", updated_at as "updated_at: _"
            FROM draft_articles
            WHERE id = $1
            "#,
            article_id
        )
        .fetch_optional(pool)
        .await?;

        match article {
            Some(article) => {
                let categories = Self::fetch_categories(pool, article.id).await?;
                Ok(Some(DraftArticleWithCategories {
                    article,
                    categories,
                }))
            }
            None => Ok(None),
        }
    }

    /// 下書き記事のカテゴリを取得
    #[instrument(skip(pool))]
    async fn fetch_categories(pool: &PgPool, article_id: Uuid) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN draft_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }
}
