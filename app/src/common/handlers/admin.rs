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
    pub is_draft: bool,
}

/// 下書き保存用入力（バリデーション緩め）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveDraftInput {
    pub id: Option<String>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
}

/// 公開記事保存用入力（バリデーション厳格）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavePublishedInput {
    pub id: String, // 公開記事は既存記事のみ
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
pub async fn get_admin_articles_handler() -> Result<Vec<AdminArticleListItem>, ServerFnError> {
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
pub async fn get_article_for_edit_handler(
    id: String,
) -> Result<Option<ArticleEditData>, ServerFnError> {
    use crate::server::contexts::AppState;
    use blog_romira_dev_cms::{DraftArticleService, PublishedArticleService};
    use uuid::Uuid;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    // まず下書きから検索
    if let Some(draft) = DraftArticleService::fetch_by_id(state.db_pool(), uuid)
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
    if let Some(published) = PublishedArticleService::fetch_by_id_for_admin(state.db_pool(), uuid)
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

/// 下書き記事の保存（新規作成または更新）
#[instrument(skip(input))]
#[server(endpoint = "admin/save_draft")]
pub async fn save_draft_handler(input: SaveDraftInput) -> Result<String, ServerFnError> {
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

/// 公開記事の保存（更新のみ）
#[instrument(skip(input))]
#[server(endpoint = "admin/save_published")]
pub async fn save_published_handler(input: SavePublishedInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use crate::server::http::response::cms_error_to_response;
    use blog_romira_dev_cms::{
        PublishedArticleService, PublishedArticleSlug, PublishedArticleTitle,
    };
    use leptos_axum::ResponseOptions;
    use uuid::Uuid;

    let response = expect_context::<ResponseOptions>();

    let title =
        PublishedArticleTitle::new(input.title).map_err(|e| cms_error_to_response(&response, e))?;
    let slug =
        PublishedArticleSlug::new(input.slug).map_err(|e| cms_error_to_response(&response, e))?;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    PublishedArticleService::update(
        state.db_pool(),
        uuid,
        &title,
        &slug,
        &input.body,
        input.description.as_deref(),
    )
    .await
    .map_err(|e| cms_error_to_response(&response, e))?;

    Ok(uuid.to_string())
}

#[instrument]
#[server(endpoint = "admin/publish_article")]
pub async fn publish_article_handler(id: String) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use crate::server::http::response::cms_error_to_response;
    use blog_romira_dev_cms::DraftArticleService;
    use leptos_axum::ResponseOptions;
    use uuid::Uuid;

    let response = expect_context::<ResponseOptions>();
    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&id).map_err(|e| ServerFnError::new(e.to_string()))?;

    let published_id = DraftArticleService::publish(state.db_pool(), uuid)
        .await
        .map_err(|e| cms_error_to_response(&response, e))?;

    Ok(published_id.to_string())
}
