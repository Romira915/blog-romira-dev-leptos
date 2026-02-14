use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// 公開記事保存用入力（バリデーション厳格）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavePublishedInput {
    pub id: String, // 公開記事は既存記事のみ
    pub title: String,
    pub slug: String,
    pub body: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    #[serde(default)]
    pub category_names: Vec<String>,
}

/// 公開記事の保存（更新のみ）
#[instrument(skip(input))]
#[server(input = Json, endpoint = "admin/save_published")]
pub async fn save_published_handler(input: SavePublishedInput) -> Result<String, ServerFnError> {
    use crate::server::contexts::AppState;
    use crate::server::http::response::cms_error_to_response;
    use blog_romira_dev_cms::{PublishedArticleSlug, PublishedArticleTitle};
    use leptos_axum::ResponseOptions;
    use uuid::Uuid;

    let response = expect_context::<ResponseOptions>();

    let title =
        PublishedArticleTitle::new(input.title).map_err(|e| cms_error_to_response(&response, e))?;
    let slug =
        PublishedArticleSlug::new(input.slug).map_err(|e| cms_error_to_response(&response, e))?;

    let state = expect_context::<AppState>();
    let uuid = Uuid::parse_str(&input.id).map_err(|e| ServerFnError::new(e.to_string()))?;

    // 旧slug取得（パージサービスが設定されている場合のみ）
    let old_article = if state.cloudflare_purge_service().is_some() {
        state
            .published_article_service()
            .fetch_by_id_for_admin(uuid)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    state
        .published_article_service()
        .update(
            uuid,
            &title,
            &slug,
            &input.body,
            input.description.as_deref(),
            input.cover_image_url.as_deref(),
        )
        .await
        .map_err(|e| cms_error_to_response(&response, e))?;

    state
        .category_service()
        .save_for_published(uuid, &input.category_names)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // CDNキャッシュパージ（ベストエフォート、未設定ならスキップ）
    if let Some(purge_service) = state.cloudflare_purge_service() {
        let mut tags = vec![
            "top-page".to_string(),
            "sitemap".to_string(),
            format!("article:{}", slug.as_str()),
        ];
        if let Some(ref old) = old_article
            && old.article.slug != slug.as_str()
        {
            tags.push(format!("article:{}", old.article.slug));
        }
        if let Err(e) = purge_service.purge_tags(&tags).await {
            tracing::warn!(error = %e, "Failed to purge Cloudflare cache after update");
        }
    }

    Ok(uuid.to_string())
}
