use super::signing::{GcsSigningService, SigningService};
use tracing::{instrument, warn};

/// GCSオブジェクト操作のエラー
#[derive(Debug, thiserror::Error)]
pub enum GcsStorageError {
    #[error("Failed to generate signed URL: {0}")]
    SigningError(String),
    #[error("Failed to delete object: {status} {body}")]
    DeleteFailed { status: u16, body: String },
    #[error("HTTP request error: {0}")]
    HttpError(String),
}

/// GCSオブジェクト操作サービス
#[derive(Clone, Debug)]
pub struct GcsStorageService {
    signing_service: GcsSigningService,
    http_client: reqwest::Client,
}

impl GcsStorageService {
    pub fn new(signing_service: GcsSigningService, http_client: reqwest::Client) -> Self {
        Self {
            signing_service,
            http_client,
        }
    }

    /// GCSからオブジェクトを削除
    #[instrument(skip(self))]
    pub async fn delete_object(&self, object_path: &str) -> Result<(), GcsStorageError> {
        // 署名付きDELETE URLを生成（有効期限15分）
        let signed_url = self
            .signing_service
            .generate_delete_url(object_path, 900)
            .await
            .map_err(|e| GcsStorageError::SigningError(e.to_string()))?;

        // DELETE リクエストを実行
        let response = self
            .http_client
            .delete(&signed_url)
            .send()
            .await
            .map_err(|e| GcsStorageError::HttpError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            // 404はオブジェクトが既に存在しない場合 → 成功として扱う
            if status.as_u16() == 404 {
                warn!(
                    object_path = object_path,
                    "GCS object not found, treating as already deleted"
                );
                return Ok(());
            }

            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            return Err(GcsStorageError::DeleteFailed {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }

    /// テスト用のスタブインスタンスを作成
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_stub() -> Self {
        Self {
            signing_service: GcsSigningService::new_stub("test-bucket".to_string()),
            http_client: reqwest::Client::new(),
        }
    }
}
