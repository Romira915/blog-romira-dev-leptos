use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 記事削除用入力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteArticleInput {
    pub id: String,
    pub is_draft: bool,
}

#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/delete_article")]
pub async fn delete_article_handler(input: DeleteArticleInput) -> Result<(), ServerFnError> {
    use crate::server::contexts::AppState;
    use crate::server::http::response::cms_error_to_response;
    use leptos_axum::ResponseOptions;
    use uuid::Uuid;

    let response = expect_context::<ResponseOptions>();
    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    if input.is_draft {
        state
            .draft_article_service()
            .delete(uuid)
            .await
            .map_err(|e| cms_error_to_response(&response, e))?;
    } else {
        state
            .published_article_service()
            .delete(uuid)
            .await
            .map_err(|e| cms_error_to_response(&response, e))?;
    }

    Ok(())
}
