use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

// DTOs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArticleEditData {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveArticleInput {
    pub id: Option<String>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminArticleListItem {
    pub id: String,
    pub title: String,
    pub is_draft: bool,
    pub published_at: Option<String>,
}

// Server functions

#[instrument]
#[server(input = GetUrl, endpoint = "admin/get_articles")]
pub async fn fetch_admin_articles() -> Result<Vec<AdminArticleListItem>, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::AdminArticleService;

    let state = expect_context::<AppState>();
    let articles = AdminArticleService::fetch_all(state.db_pool())
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

#[instrument]
#[server(input = GetUrl, endpoint = "admin/get_article")]
pub async fn fetch_article_for_edit(id: String) -> Result<Option<ArticleEditData>, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::DraftArticleService;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let article = DraftArticleService::fetch_by_id(state.db_pool(), uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(article.map(|a| ArticleEditData {
        id: a.article.id.to_string(),
        title: a.article.title,
        slug: a.article.slug,
        body: a.article.body,
        description: a.article.description,
    }))
}

#[instrument(skip(input))]
#[server(endpoint = "admin/save_article")]
pub async fn save_article_action(input: SaveArticleInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::DraftArticleService;
    use uuid::Uuid;

    let state = expect_context::<AppState>();

    let article_id = match input.id {
        Some(id) => {
            let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;
            DraftArticleService::update(
                state.db_pool(),
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
        None => DraftArticleService::create(
            state.db_pool(),
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

#[instrument]
#[server(endpoint = "admin/publish_article")]
pub async fn publish_article_action(id: String) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::DraftArticleService;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let published_id = DraftArticleService::publish(state.db_pool(), uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(published_id.to_string())
}
