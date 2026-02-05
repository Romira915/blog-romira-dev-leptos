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

/// 画像を削除
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/images/delete")]
pub async fn delete_image_handler(input: DeleteImageInput) -> Result<(), ServerFnError> {
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let service = state.image_service();

    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    service
        .delete(uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}
