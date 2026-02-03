use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// アップロードURL生成用入力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateUploadUrlInput {
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
}

/// アップロードURL生成レスポンス
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateUploadUrlResponse {
    /// 署名付きアップロードURL
    pub upload_url: String,
    /// GCS上のオブジェクトパス（DB登録時に使用）
    pub gcs_path: String,
}

/// 署名付きアップロードURLを生成
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/images/upload-url")]
pub async fn generate_upload_url_handler(
    input: GenerateUploadUrlInput,
) -> Result<GenerateUploadUrlResponse, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::ImageService;

    let state = expect_context::<AppState>();

    // バリデーション
    ImageService::validate_mime_type(&input.content_type)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    ImageService::validate_file_size(input.size_bytes)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let gcs_signing_service = state.gcs_signing_service();

    let gcs_path = ImageService::generate_gcs_path(&input.filename);

    // 署名付きURL生成（有効期限: 15分）
    let upload_url = gcs_signing_service
        .generate_upload_url(&gcs_path, &input.content_type, 15 * 60)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(GenerateUploadUrlResponse {
        upload_url,
        gcs_path,
    })
}
