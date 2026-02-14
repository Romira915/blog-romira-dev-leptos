use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 下書き公開用入力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishArticleInput {
    pub id: String,
}

#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/publish_article")]
pub async fn publish_article_handler(input: PublishArticleInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use crate::server::http::response::cms_error_to_response;
    use leptos_axum::ResponseOptions;
    use uuid::Uuid;

    let response = expect_context::<ResponseOptions>();
    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let published_id = state
        .draft_article_service()
        .publish(uuid)
        .await
        .map_err(|e| cms_error_to_response(&response, e))?;

    // CDNキャッシュパージ（ベストエフォート、未設定ならスキップ）
    if let Some(purge_service) = state.cloudflare_purge_service() {
        let tags = vec!["top-page".to_string()];
        if let Err(e) = purge_service.purge_tags(&tags).await {
            tracing::warn!(error = %e, "Failed to purge Cloudflare cache after publish");
        }
    }

    Ok(published_id.to_string())
}
