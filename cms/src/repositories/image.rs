use crate::error::CmsError;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 画像リポジトリ（CUD操作）
pub struct ImageRepository;

impl ImageRepository {
    /// 画像を作成
    #[instrument(skip(pool))]
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        filename: &str,
        gcs_path: &str,
        mime_type: &str,
        size_bytes: i64,
        width: Option<i32>,
        height: Option<i32>,
        alt_text: Option<&str>,
        now: NaiveDateTime,
    ) -> Result<Uuid, CmsError> {
        let image_id = sqlx::query_scalar!(
            r#"
            INSERT INTO images (filename, gcs_path, mime_type, size_bytes, width, height, alt_text, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            filename,
            gcs_path,
            mime_type,
            size_bytes,
            width,
            height,
            alt_text,
            now as _
        )
        .fetch_one(pool)
        .await?;

        Ok(image_id)
    }

    /// 画像を削除
    #[instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, image_id: Uuid) -> Result<(), CmsError> {
        let rows = sqlx::query!("DELETE FROM images WHERE id = $1", image_id)
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
    use crate::test_utils::*;

    #[sqlx::test]
    async fn test_createで画像が作成されること(pool: PgPool) {
        let now = utc_now();
        let image_id = ImageRepository::create(
            &pool,
            "test.jpg",
            "images/test.jpg",
            "image/jpeg",
            1024,
            Some(800),
            Some(600),
            Some("テスト画像"),
            now,
        )
        .await
        .expect("Failed to create image");

        let image = sqlx::query!(
            r#"SELECT filename, gcs_path, mime_type, size_bytes, width, height, alt_text FROM images WHERE id = $1"#,
            image_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch image");

        assert_eq!(image.filename, "test.jpg");
        assert_eq!(image.gcs_path, "images/test.jpg");
        assert_eq!(image.mime_type, "image/jpeg");
        assert_eq!(image.size_bytes, 1024);
        assert_eq!(image.width, Some(800));
        assert_eq!(image.height, Some(600));
        assert_eq!(image.alt_text, Some("テスト画像".to_string()));
    }

    #[sqlx::test]
    async fn test_createでサイズなし画像が作成されること(pool: PgPool) {
        let now = utc_now();
        let image_id = ImageRepository::create(
            &pool,
            "test.jpg",
            "images/test.jpg",
            "image/jpeg",
            1024,
            None,
            None,
            None,
            now,
        )
        .await
        .expect("Failed to create image");

        let image = sqlx::query!(
            r#"SELECT width, height, alt_text FROM images WHERE id = $1"#,
            image_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch image");

        assert!(image.width.is_none());
        assert!(image.height.is_none());
        assert!(image.alt_text.is_none());
    }

    #[sqlx::test]
    async fn test_deleteで画像が削除されること(pool: PgPool) {
        let image_id =
            insert_test_image(&pool, "test.jpg", "images/test.jpg", "image/jpeg", 1024).await;

        ImageRepository::delete(&pool, image_id)
            .await
            .expect("Failed to delete image");

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM images WHERE id = $1"#,
            image_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to count images");

        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn test_存在しない画像をdeleteするとnotfoundエラーになること(
        pool: PgPool,
    ) {
        let nonexistent_id = Uuid::now_v7();
        let result = ImageRepository::delete(&pool, nonexistent_id).await;

        assert!(matches!(result, Err(CmsError::NotFound)));
    }
}
