use crate::error::CmsError;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 下書き記事リポジトリ（CUD操作）
pub struct DraftArticleRepository;

impl DraftArticleRepository {
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

    /// 下書き記事をUpsert（存在しなければINSERT、存在すればUPDATE）
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(pool))]
    pub async fn upsert(
        pool: &PgPool,
        id: Uuid,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
        cover_image_url: Option<&str>,
        now: NaiveDateTime,
    ) -> Result<(), CmsError> {
        sqlx::query!(
            r#"
            INSERT INTO draft_articles (id, slug, title, body, description, cover_image_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            ON CONFLICT (id) DO UPDATE SET
                slug = EXCLUDED.slug,
                title = EXCLUDED.title,
                body = EXCLUDED.body,
                description = EXCLUDED.description,
                cover_image_url = EXCLUDED.cover_image_url,
                updated_at = EXCLUDED.updated_at
            "#,
            id,
            slug,
            title,
            body,
            description,
            cover_image_url,
            now as _
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::utc_now;

    #[sqlx::test]
    async fn test_deleteで記事が削除されること(pool: PgPool) {
        let now = utc_now();
        let id = Uuid::now_v7();
        DraftArticleRepository::upsert(&pool, id, "削除対象", "to-delete", "本文", None, None, now)
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
        let nonexistent_id = Uuid::now_v7();
        let result = DraftArticleRepository::delete(&pool, nonexistent_id).await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }

    #[sqlx::test]
    async fn test_upsertで新規記事が作成されること(pool: PgPool) {
        let now = utc_now();
        let id = Uuid::now_v7();

        DraftArticleRepository::upsert(
            &pool,
            id,
            "テスト記事タイトル",
            "test-article-slug",
            "これはテスト記事の本文です。",
            Some("テスト記事の説明"),
            None,
            now,
        )
        .await
        .expect("Failed to upsert draft article");

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
    async fn test_upsertで既存記事が更新されること(pool: PgPool) {
        let now = utc_now();
        let id = Uuid::now_v7();

        // 1回目のupsert（作成）
        DraftArticleRepository::upsert(
            &pool,
            id,
            "元のタイトル",
            "original",
            "元の本文",
            None,
            None,
            now,
        )
        .await
        .expect("Failed to create");

        // 2回目のupsert（更新）
        DraftArticleRepository::upsert(
            &pool,
            id,
            "更新後のタイトル",
            "updated",
            "更新後の本文",
            Some("更新後の説明"),
            None,
            utc_now(),
        )
        .await
        .expect("Failed to update");

        let article = sqlx::query!(
            r#"SELECT title, slug, body, description FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(article.title, "更新後のタイトル");
        assert_eq!(article.slug, "updated");
        assert_eq!(article.body, "更新後の本文");
        assert_eq!(article.description, Some("更新後の説明".to_string()));
    }

    #[sqlx::test]
    async fn test_upsertで連打しても1件だけ存在すること(pool: PgPool) {
        let now = utc_now();
        let id = Uuid::now_v7();

        // 3回連続でupsert（連打シミュレーション）
        for i in 0..3 {
            let title = format!("タイトル{}", i);
            DraftArticleRepository::upsert(
                &pool,
                id,
                title.as_str(),
                "slug",
                "本文",
                None,
                None,
                now,
            )
            .await
            .expect("Failed to upsert");
        }

        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to count");

        assert_eq!(count, 1);

        // 最後の更新内容が反映されていること
        let article = sqlx::query!(r#"SELECT title FROM draft_articles WHERE id = $1"#, id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch");

        assert_eq!(article.title, "タイトル2");
    }

    #[sqlx::test]
    async fn test_upsertでcover_image_urlが保存されること(pool: PgPool) {
        let now = utc_now();
        let id = Uuid::now_v7();

        DraftArticleRepository::upsert(
            &pool,
            id,
            "カバー画像テスト",
            "cover-test",
            "本文",
            None,
            Some("https://example.com/image.jpg"),
            now,
        )
        .await
        .expect("Failed to upsert");

        let article = sqlx::query!(
            r#"SELECT cover_image_url FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(
            article.cover_image_url.as_deref(),
            Some("https://example.com/image.jpg")
        );

        // 更新でcover_image_urlをNoneに変更
        DraftArticleRepository::upsert(
            &pool,
            id,
            "カバー画像テスト",
            "cover-test",
            "本文",
            None,
            None,
            utc_now(),
        )
        .await
        .expect("Failed to update");

        let article = sqlx::query!(
            r#"SELECT cover_image_url FROM draft_articles WHERE id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(article.cover_image_url, None);
    }
}
