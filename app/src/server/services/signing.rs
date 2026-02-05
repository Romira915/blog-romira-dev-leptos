use std::sync::Arc;
use std::time::Duration;

use google_cloud_auth::credentials::service_account::Builder as ServiceAccountBuilder;
use google_cloud_auth::signer::Signer;
use google_cloud_storage::builder::storage::SignedUrlBuilder;
use tracing::instrument;

/// 署名付きURL生成のエラー
#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("Failed to create signer: {0}")]
    SignerCreation(String),
    #[error("Failed to generate signed URL: {0}")]
    SigningError(String),
}

/// 署名付きURL生成サービスのトレイト
#[allow(dead_code)]
pub trait SigningService {
    async fn generate_upload_url(
        &self,
        object_path: &str,
        content_type: &str,
        duration_secs: u64,
    ) -> Result<String, SigningError>;

    fn bucket(&self) -> &str;
}

/// GCS署名付きURL生成サービス
#[derive(Clone, Debug)]
pub struct GcsSigningService {
    bucket: String,
    signer: Option<Arc<Signer>>,
}

impl GcsSigningService {
    /// サービスアカウントキーJSONから新しいGcsSigningServiceを作成
    pub fn from_service_account_key(
        bucket: String,
        service_account_key_json: &str,
    ) -> Result<Self, SigningError> {
        let service_account_key: serde_json::Value = serde_json::from_str(service_account_key_json)
            .map_err(|e| SigningError::SignerCreation(e.to_string()))?;

        let signer = ServiceAccountBuilder::new(service_account_key)
            .build_signer()
            .map_err(|e| SigningError::SignerCreation(e.to_string()))?;

        Ok(Self {
            bucket,
            signer: Some(Arc::new(signer)),
        })
    }

    /// テスト用のスタブインスタンスを作成
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_stub(bucket: String) -> Self {
        Self {
            bucket,
            signer: None,
        }
    }
}

impl SigningService for GcsSigningService {
    /// アップロード用の署名付きURLを生成 (PUT)
    ///
    /// NOTE: google-cloud-storageのSignedUrlBuilderが内部で非Send型を使用するため、
    /// spawn_blockingでラップしてLeptos server functionとの互換性を確保
    #[instrument(skip(self))]
    async fn generate_upload_url(
        &self,
        object_path: &str,
        content_type: &str,
        duration_secs: u64,
    ) -> Result<String, SigningError> {
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| SigningError::SigningError("Signer not configured".to_string()))?;

        let bucket_resource = format!("projects/_/buckets/{}", self.bucket);
        let object_path = object_path.to_string();
        let content_type = content_type.to_string();
        let signer = Arc::clone(signer);

        // spawn_blockingで非Send問題を回避
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async {
                SignedUrlBuilder::for_object(&bucket_resource, &object_path)
                    .with_method(http::Method::PUT)
                    .with_expiration(Duration::from_secs(duration_secs))
                    .with_header("content-type", &content_type)
                    .sign_with(&signer)
                    .await
            })
        })
        .await
        .map_err(|e| SigningError::SigningError(format!("Task join error: {}", e)))?
        .map_err(|e| SigningError::SigningError(e.to_string()))
    }

    /// バケット名を取得
    fn bucket(&self) -> &str {
        self.bucket.as_str()
    }
}
