use crate::error::CmsError;
use crate::models::Category;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// カテゴリクエリサービス
pub struct CategoryQuery;

impl CategoryQuery {
    /// 全カテゴリを取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"SELECT id, name, slug FROM categories ORDER BY name"#
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    /// 公開記事のカテゴリを取得
    #[instrument(skip(pool))]
    pub async fn fetch_for_published(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Vec<Category>, CmsError> {
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

    /// 下書き記事のカテゴリを取得
    #[instrument(skip(pool))]
    pub async fn fetch_for_draft(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Vec<Category>, CmsError> {
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
