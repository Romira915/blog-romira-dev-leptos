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

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn utc_now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    #[sqlx::test]
    async fn test_createで記事が作成されること(pool: PgPool) {
        let now = utc_now();
        let id = DraftArticleRepository::create(
            &pool,
            "テスト記事タイトル",
            "test-article-slug",
            "これはテスト記事の本文です。",
            Some("テスト記事の説明"),
            now,
        )
        .await
        .expect("Failed to create draft article");

        assert!(!id.is_nil());

        let article = sqlx::query!(
            r#"SELECT title, slug, body, description FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch created article");

        assert_eq!(article.title, "テスト記事タイトル");
        assert_eq!(article.slug, "test-article-slug");
        assert_eq!(article.body, "これはテスト記事の本文です。");
        assert_eq!(article.description.as_deref(), Some("テスト記事の説明"));
    }

    #[sqlx::test]
    async fn test_descriptionがnoneでも記事が作成されること(pool: PgPool) {
        let now = utc_now();
        let id =
            DraftArticleRepository::create(&pool, "説明なし記事", "no-desc", "本文のみ", None, now)
                .await
                .expect("Failed to create draft article without description");

        let article = sqlx::query!(
            r#"SELECT description FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch created article");

        assert!(article.description.is_none());
    }

    #[sqlx::test]
    async fn test_updateで記事が更新されること(pool: PgPool) {
        let now = utc_now();
        let id = DraftArticleRepository::create(
            &pool,
            "元のタイトル",
            "original",
            "元の本文",
            Some("元の説明"),
            now,
        )
        .await
        .expect("Failed to create draft article");

        DraftArticleRepository::update(
            &pool,
            id,
            "更新後のタイトル",
            "updated",
            "更新後の本文",
            Some("更新後の説明"),
            utc_now(),
        )
        .await
        .expect("Failed to update draft article");

        let article = sqlx::query!(
            r#"SELECT title, slug, body, description FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated article");

        assert_eq!(article.title, "更新後のタイトル");
        assert_eq!(article.slug, "updated");
        assert_eq!(article.body, "更新後の本文");
        assert_eq!(article.description, Some("更新後の説明".to_string()));
    }

    #[sqlx::test]
    async fn test_存在しない記事をupdateするとnotfoundエラーになること(
        pool: PgPool,
    ) {
        let nonexistent_id = Uuid::new_v4();
        let result = DraftArticleRepository::update(
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

    #[sqlx::test]
    async fn test_deleteで記事が削除されること(pool: PgPool) {
        let now = utc_now();
        let id = DraftArticleRepository::create(&pool, "削除対象", "to-delete", "本文", None, now)
            .await
            .expect("Failed to create draft article");

        let exists_before = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM draft_articles WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check existence");
        assert!(exists_before);

        DraftArticleRepository::delete(&pool, id)
            .await
            .expect("Failed to delete draft article");

        let exists_after = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM draft_articles WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check existence");
        assert!(!exists_after);
    }

    #[sqlx::test]
    async fn test_存在しない記事をdeleteするとnotfoundエラーになること(
        pool: PgPool,
    ) {
        let nonexistent_id = Uuid::new_v4();
        let result = DraftArticleRepository::delete(&pool, nonexistent_id).await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }
}
