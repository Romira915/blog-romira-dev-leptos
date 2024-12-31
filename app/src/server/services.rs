use crate::error::HomePageError;
use crate::server::models::newt_article::NewtArticleCollection;
use crate::{constants, SERVER_CONFIG};

pub(crate) async fn get_newt_articles(
    client: reqwest::Client,
    is_preview: bool,
) -> Result<NewtArticleCollection, HomePageError> {
    let (base_url, api_token) = if is_preview {
        (constants::NEWT_BASE_URL, &SERVER_CONFIG.newt_api_token)
    } else {
        (
            constants::NEWT_CDN_BASE_URL,
            &SERVER_CONFIG.newt_cdn_api_token,
        )
    };

    let response = client
        .get(format!("{base_url}/blog/article"))
        .bearer_auth(api_token)
        .send()
        .await?;

    let articles: NewtArticleCollection = response.json().await?;

    Ok(articles)
}
