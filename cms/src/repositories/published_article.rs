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
}
