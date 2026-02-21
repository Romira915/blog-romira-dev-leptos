use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

/// 画像削除用入力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteImageInput {
    pub id: String,
}

/// 画像を削除（DB + GCS）
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/images/delete")]
pub async fn delete_image_handler(input: DeleteImageInput) -> Result<(), ServerFnError> {
    use crate::server::contexts::AppState;
    use tracing::warn;

    let state = expect_context::<AppState>();
    let image_service = state.image_service();
    let gcs_storage_service = state.gcs_storage_service();

    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    // 画像のGCSパスを取得
    let image = image_service
        .fetch_by_id(uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("画像が見つかりません".to_string()))?;

    // GCSからオブジェクトを削除（失敗してもDB削除は続行）
    if let Err(e) = gcs_storage_service.delete_object(&image.gcs_path).await {
        warn!(
            image_id = %uuid,
            gcs_path = %image.gcs_path,
            error = %e,
            "GCSからの画像削除に失敗しました"
        );
    }

    // DBから画像レコードを削除
    image_service
        .delete(uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
