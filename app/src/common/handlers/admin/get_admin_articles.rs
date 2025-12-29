use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminArticleListItem {
    pub id: String,
    pub title: String,
    pub is_draft: bool,
    pub published_at: Option<String>,
}

#[instrument]
#[server(input = GetUrl, endpoint = "admin/get_articles")]
pub async fn get_admin_articles_handler() -> Result<Vec<AdminArticleListItem>, ServerFnError> {
    use crate::server::contexts::AppState;

    let state = expect_context::<AppState>();
    let service = state.admin_article_service();
    let articles = service
        .fetch_all()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(articles
        .into_iter()
        .map(|a| AdminArticleListItem {
            id: a.id().to_string(),
            title: a.title().to_string(),
            is_draft: a.is_draft(),
            published_at: a
                .published_at()
                .map(|d| d.format("%Y年%m月%d日").to_string()),
        })
        .collect())
}
