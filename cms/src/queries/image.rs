use crate::error::CmsError;
use crate::models::Image;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 画像クエリサービス（SELECT操作）
pub struct ImageQuery;

impl ImageQuery {
    /// 全画像を作成日時降順で取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<Image>, CmsError> {
        let images = sqlx::query_as!(
            Image,
            r#"
            SELECT id, filename, gcs_path, mime_type, size_bytes, width, height, alt_text,
                   created_at as "created_at: _"
            FROM images
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(images)
    }

    /// 画像をIDで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_id(pool: &PgPool, image_id: Uuid) -> Result<Option<Image>, CmsError> {
        let image = sqlx::query_as!(
            Image,
            r#"
            SELECT id, filename, gcs_path, mime_type, size_bytes, width, height, alt_text,
                   created_at as "created_at: _"
            FROM images
            WHERE id = $1
            "#,
            image_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(image)
    }

    /// 指定したGCSパスが既に存在するかチェック
    #[instrument(skip(pool))]
    pub async fn exists_by_gcs_path(pool: &PgPool, gcs_path: &str) -> Result<bool, CmsError> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM images WHERE gcs_path = $1) as "exists!: bool""#,
            gcs_path
        )
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[sqlx::test]
    async fn test_fetch_allで画像一覧が取得されること(pool: PgPool) {
        insert_test_image(&pool, "test1.jpg", "images/test1.jpg", "image/jpeg", 1024).await;
        insert_test_image(&pool, "test2.png", "images/test2.png", "image/png", 2048).await;

        let result = ImageQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 2);
    }

    #[sqlx::test]
    async fn test_fetch_by_idで画像が取得されること(pool: PgPool) {
        let image_id =
            insert_test_image(&pool, "test.jpg", "images/test.jpg", "image/jpeg", 1024).await;

        let result = ImageQuery::fetch_by_id(&pool, image_id)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_some());
        let image = result.unwrap();
        assert_eq!(image.id, image_id);
        assert_eq!(image.filename, "test.jpg");
    }

    #[sqlx::test]
    async fn test_存在しないidでfetch_by_idするとnoneが返ること(pool: PgPool) {
        let nonexistent_id = Uuid::now_v7();

        let result = ImageQuery::fetch_by_id(&pool, nonexistent_id)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_exists_by_gcs_pathで存在確認ができること(pool: PgPool) {
        insert_test_image(&pool, "test.jpg", "images/test.jpg", "image/jpeg", 1024).await;

        let exists = ImageQuery::exists_by_gcs_path(&pool, "images/test.jpg")
            .await
            .expect("Failed to check existence");
        assert!(exists);

        let not_exists = ImageQuery::exists_by_gcs_path(&pool, "images/nonexistent.jpg")
            .await
            .expect("Failed to check existence");
        assert!(!not_exists);
    }
}
