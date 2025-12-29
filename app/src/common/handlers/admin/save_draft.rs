use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 下書き保存用入力（バリデーション緩め）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveDraftInput {
    pub id: Option<String>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

/// 下書き記事の保存（新規作成または更新）
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/save_draft")]
pub async fn save_draft_handler(input: SaveDraftInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let service = state.draft_article_service();

    let article_id = match input.id {
        Some(id) => {
            let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
            service
                .update(
                    uuid,
                    &input.title,
                    &input.slug,
                    &input.body,
                    input.description.as_deref(),
                )
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            uuid
        }
        None => service
            .create(
                &input.title,
                &input.slug,
                &input.body,
                input.description.as_deref(),
            )
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?,
    };

    Ok(article_id.to_string())
}
