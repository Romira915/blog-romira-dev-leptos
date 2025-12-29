use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleEditData {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
    pub is_draft: bool,
}

#[instrument]
#[server(input = GetUrl, endpoint = "admin/get_article")]
pub async fn get_article_for_edit_handler(
    id: String,
) -> Result<Option<ArticleEditData>, ServerFnError> {
    use crate::server::contexts::AppState;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let draft_service = state.draft_article_service();
    let published_service = state.published_article_service();

    // まず下書きから検索
    if let Some(draft) = draft_service
        .fetch_by_id(uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Ok(Some(ArticleEditData {
            id: draft.article.id.to_string(),
            title: draft.article.title,
            slug: draft.article.slug,
            body: draft.article.body,
            description: draft.article.description,
            is_draft: true,
        }));
    }

    // 下書きになければ公開記事から検索
    if let Some(published) = published_service
        .fetch_by_id_for_admin(uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        return Ok(Some(ArticleEditData {
            id: published.article.id.to_string(),
            title: published.article.title,
            slug: published.article.slug,
            body: published.article.body,
            description: published.article.description,
            is_draft: false,
        }));
    }

    Ok(None)
}
