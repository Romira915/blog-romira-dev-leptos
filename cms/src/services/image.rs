use crate::error::CmsError;
use crate::models::Image;
use crate::queries::ImageQuery;
use crate::repositories::ImageRepository;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use super::utc_now;

/// 許可されたMIMEタイプ
const ALLOWED_MIME_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];

/// 最大ファイルサイズ (10MB)
const MAX_FILE_SIZE: i64 = 10 * 1024 * 1024;

/// 画像サービス
#[derive(Debug, Clone)]
pub struct ImageService {
    pool: PgPool,
    path_prefix: String,
}

impl ImageService {
    pub fn new(pool: PgPool, path_prefix: String) -> Self {
        Self { pool, path_prefix }
    }

    /// 全画像を取得
    #[instrument(skip(self))]
    pub async fn fetch_all(&self) -> Result<Vec<Image>, CmsError> {
        ImageQuery::fetch_all(&self.pool).await
    }

    /// 画像をIDで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_id(&self, image_id: Uuid) -> Result<Option<Image>, CmsError> {
        ImageQuery::fetch_by_id(&self.pool, image_id).await
    }

    /// MIMEタイプをバリデーション
    pub fn validate_mime_type(mime_type: &str) -> Result<(), CmsError> {
        if !ALLOWED_MIME_TYPES.contains(&mime_type) {
            return Err(CmsError::ValidationError(format!(
                "許可されていないファイル形式です: {}。許可: {:?}",
                mime_type, ALLOWED_MIME_TYPES
            )));
        }
        Ok(())
    }

    /// ファイルサイズをバリデーション
    pub fn validate_file_size(size_bytes: i64) -> Result<(), CmsError> {
        if size_bytes > MAX_FILE_SIZE {
            return Err(CmsError::ValidationError(format!(
                "ファイルサイズが大きすぎます: {}MB。最大: {}MB",
                size_bytes / 1024 / 1024,
                MAX_FILE_SIZE / 1024 / 1024
            )));
        }
        if size_bytes <= 0 {
            return Err(CmsError::ValidationError(
                "ファイルサイズが不正です".to_string(),
            ));
        }
        Ok(())
    }

    /// ファイル名からGCSオブジェクトパスを生成（prefix + UUID v7 + 元のファイル名）
    pub fn generate_gcs_path(&self, filename: &str) -> String {
        let uuid = Uuid::now_v7();
        format!("{}/images/{}/{}", self.path_prefix, uuid, filename)
    }

    /// 画像を登録（バリデーション付き）
    #[instrument(skip(self))]
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        filename: &str,
        gcs_path: &str,
        mime_type: &str,
        size_bytes: i64,
        width: Option<i32>,
        height: Option<i32>,
        alt_text: Option<&str>,
    ) -> Result<Uuid, CmsError> {
        // バリデーション
        Self::validate_mime_type(mime_type)?;
        Self::validate_file_size(size_bytes)?;

        // GCSパスの重複チェック
        if ImageQuery::exists_by_gcs_path(&self.pool, gcs_path).await? {
            return Err(CmsError::ValidationError(
                "この画像は既に登録されています".to_string(),
            ));
        }

        ImageRepository::create(
            &self.pool,
            filename,
            gcs_path,
            mime_type,
            size_bytes,
            width,
            height,
            alt_text,
            utc_now(),
        )
        .await
    }

    /// 画像を削除
    #[instrument(skip(self))]
    pub async fn delete(&self, image_id: Uuid) -> Result<(), CmsError> {
        ImageRepository::delete(&self.pool, image_id).await
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_validate_mime_typeで許可されたタイプが通ること() {
        assert!(ImageService::validate_mime_type("image/jpeg").is_ok());
        assert!(ImageService::validate_mime_type("image/png").is_ok());
        assert!(ImageService::validate_mime_type("image/gif").is_ok());
        assert!(ImageService::validate_mime_type("image/webp").is_ok());
    }

    #[test]
    fn test_validate_mime_typeで許可されていないタイプがエラーになること() {
        let result = ImageService::validate_mime_type("image/bmp");
        assert!(matches!(result, Err(CmsError::ValidationError(_))));

        let result = ImageService::validate_mime_type("text/plain");
        assert!(matches!(result, Err(CmsError::ValidationError(_))));
    }

    #[test]
    fn test_validate_file_sizeで適切なサイズが通ること() {
        assert!(ImageService::validate_file_size(1024).is_ok());
        assert!(ImageService::validate_file_size(MAX_FILE_SIZE).is_ok());
    }

    #[test]
    fn test_validate_file_sizeで大きすぎるファイルがエラーになること() {
        let result = ImageService::validate_file_size(MAX_FILE_SIZE + 1);
        assert!(matches!(result, Err(CmsError::ValidationError(_))));
    }

    #[test]
    fn test_validate_file_sizeで不正なサイズがエラーになること() {
        let result = ImageService::validate_file_size(0);
        assert!(matches!(result, Err(CmsError::ValidationError(_))));

        let result = ImageService::validate_file_size(-1);
        assert!(matches!(result, Err(CmsError::ValidationError(_))));
    }

    #[sqlx::test]
    async fn test_generate_gcs_pathでprefix付きuuidパスが生成されること(pool: PgPool) {
        let service = ImageService::new(pool, "dev".to_string());
        let path = service.generate_gcs_path("test.jpg");
        assert!(path.starts_with("dev/images/"));
        assert!(path.ends_with("/test.jpg"));
        // {prefix}/images/{uuid}/{filename} の4パート
        let parts: Vec<&str> = path.split('/').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "dev");
        assert_eq!(parts[1], "images");
        assert!(Uuid::parse_str(parts[2]).is_ok());
        assert_eq!(parts[3], "test.jpg");
    }

    #[sqlx::test]
    async fn test_createで画像が作成されること(pool: PgPool) {
        let service = ImageService::new(pool.clone(), "test".to_string());

        let image_id = service
            .create(
                "test.jpg",
                "images/test.jpg",
                "image/jpeg",
                1024,
                Some(800),
                Some(600),
                Some("テスト画像"),
            )
            .await
            .expect("Failed to create image");

        let image = service
            .fetch_by_id(image_id)
            .await
            .expect("Failed to fetch")
            .expect("Image not found");

        assert_eq!(image.filename, "test.jpg");
    }

    #[sqlx::test]
    async fn test_createで重複gcs_pathがエラーになること(pool: PgPool) {
        let service = ImageService::new(pool.clone(), "test".to_string());

        service
            .create(
                "test.jpg",
                "images/test.jpg",
                "image/jpeg",
                1024,
                None,
                None,
                None,
            )
            .await
            .expect("Failed to create first image");

        let result = service
            .create(
                "test2.jpg",
                "images/test.jpg", // 同じパス
                "image/jpeg",
                2048,
                None,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(CmsError::ValidationError(_))));
    }

    #[sqlx::test]
    async fn test_deleteで画像が削除されること(pool: PgPool) {
        let image_id =
            insert_test_image(&pool, "test.jpg", "images/test.jpg", "image/jpeg", 1024).await;

        let service = ImageService::new(pool.clone(), "test".to_string());
        service
            .delete(image_id)
            .await
            .expect("Failed to delete image");

        let result = service
            .fetch_by_id(image_id)
            .await
            .expect("Failed to fetch");
        assert!(result.is_none());
    }
}
