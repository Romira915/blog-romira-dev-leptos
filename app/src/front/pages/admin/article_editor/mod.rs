mod state;
mod view;

pub use view::ArticleEditorPage;

// Server functions and DTOs
use leptos::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ArticleEditData {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SaveArticleInput {
    pub id: Option<String>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

#[server(endpoint = "admin/get_article")]
pub async fn fetch_article_for_edit(id: String) -> Result<Option<ArticleEditData>, ServerFnError> {
    use blog_romira_dev_cms::DraftArticleService;
    use crate::server::contexts::AppState;
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

#[server(endpoint = "admin/save_article")]
pub async fn save_article_action(input: SaveArticleInput) -> Result<String, ServerFnError> {
    use blog_romira_dev_cms::DraftArticleService;
    use crate::server::contexts::AppState;
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
        None => {
            DraftArticleService::create(
                state.db_pool(),
                &input.title,
                &input.slug,
                &input.body,
                input.description.as_deref(),
            )
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        }
    };

    Ok(article_id.to_string())
}

#[server(endpoint = "admin/publish_article")]
pub async fn publish_article_action(id: String) -> Result<String, ServerFnError> {
    use blog_romira_dev_cms::DraftArticleService;
    use crate::server::contexts::AppState;
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let published_id = DraftArticleService::publish(state.db_pool(), uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(published_id.to_string())
}
