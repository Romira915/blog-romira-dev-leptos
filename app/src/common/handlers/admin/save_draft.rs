use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 下書き保存用入力（idは必須）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveDraftInput {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

/// 下書き記事の保存（Upsert: 存在しなければ作成、存在すれば更新）
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/save_draft")]
pub async fn save_draft_handler(input: SaveDraftInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let service = state.draft_article_service();

    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    service
        .save(
            uuid,
            &input.title,
            &input.slug,
            &input.body,
            input.description.as_deref(),
            input.cover_image_url.as_deref(),
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(uuid.to_string())
}
