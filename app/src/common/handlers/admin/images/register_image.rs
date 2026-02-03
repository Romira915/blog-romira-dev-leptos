use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 画像登録用入力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterImageInput {
    pub filename: String,
    pub gcs_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
}

/// 画像登録レスポンス
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterImageResponse {
    pub id: String,
    pub imgix_url: String,
}

/// 画像をDBに登録
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/images")]
pub async fn register_image_handler(
    input: RegisterImageInput,
) -> Result<RegisterImageResponse, ServerFnError> {
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let service = state.image_service();

    let image_id = service
        .create(
            &input.filename,
            &input.gcs_path,
            &input.mime_type,
            input.size_bytes,
            input.width,
            input.height,
            input.alt_text.as_deref(),
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // imgix URL を生成
    let imgix_url = state.imgix_service().generate_url(&input.gcs_path);

    Ok(RegisterImageResponse {
        id: image_id.to_string(),
        imgix_url,
    })
}
