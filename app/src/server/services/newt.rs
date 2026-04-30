use crate::SERVER_CONFIG;
use crate::error::NewtArticleServiceError;
use crate::server::models::newt_article::NewtArticle;
use crate::server::models::newt_author::Author;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Clone)]
pub(crate) struct NewtArticleService {
    client: reqwest::Client,
    newt_cdn_base_url: Arc<String>,
    newt_base_url: Arc<String>,
}

impl NewtArticleService {
    #[instrument]
    pub(crate) fn new(
        client: reqwest::Client,
        newt_cdn_base_url: impl ToString + Debug,
        newt_base_url: impl ToString + Debug,
    ) -> Self {
        Self {
            client,
            newt_cdn_base_url: Arc::new(newt_cdn_base_url.to_string()),
            newt_base_url: Arc::new(newt_base_url.to_string()),
        }
    }

    #[instrument]
    async fn fetch_article<T>(
        &self,
        article_id: T,
        is_preview: bool,
    ) -> Result<Option<NewtArticle>, NewtArticleServiceError>
    where
        T: std::fmt::Display + Debug,
    {
        let (base_url, api_token) = if is_preview {
            (&self.newt_base_url, &SERVER_CONFIG.newt_api_token)
        } else {
            (&self.newt_cdn_base_url, &SERVER_CONFIG.newt_cdn_api_token)
        };

        let response = self
            .client
            .get(format!("{base_url}/blog/article/{article_id}"))
            .bearer_auth(api_token)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(NewtArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let article: NewtArticle = response.json().await?;

        Ok(Some(article))
    }

    #[instrument]
    pub(crate) async fn fetch_preview_article<T>(
        &self,
        article_id: T,
    ) -> Result<Option<NewtArticle>, NewtArticleServiceError>
    where
        T: std::fmt::Display + Debug,
    {
        self.fetch_article(article_id, true).await
    }

    #[instrument]
    pub(crate) async fn fetch_author<T>(
        &self,
        author_id: T,
    ) -> Result<Author, NewtArticleServiceError>
    where
        T: std::fmt::Display + Debug,
    {
        let (base_url, api_token) = (&self.newt_cdn_base_url, &SERVER_CONFIG.newt_cdn_api_token);

        let response = self
            .client
            .get(format!("{base_url}/blog/author/{author_id}"))
            .bearer_auth(api_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NewtArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let author: Author = response.json().await?;

        Ok(author)
    }
}
