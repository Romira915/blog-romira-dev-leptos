use crate::error::CmsError;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 下書き記事リポジトリ（CUD操作）
pub struct DraftArticleRepository;

impl DraftArticleRepository {
    /// 下書き記事を作成
    #[instrument(skip(pool))]
    pub async fn create(
        pool: &PgPool,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
        now: NaiveDateTime,
    ) -> Result<Uuid, CmsError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO draft_articles (slug, title, body, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING id
            "#,
            slug,
            title,
            body,
            description,
            now as _
        )
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    /// 下書き記事を更新
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
            UPDATE draft_articles
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

    /// 下書き記事を削除
    #[instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, article_id: Uuid) -> Result<(), CmsError> {
        let rows = sqlx::query!("DELETE FROM draft_articles WHERE id = $1", article_id)
            .execute(pool)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(CmsError::NotFound);
        }

        Ok(())
    }
}
