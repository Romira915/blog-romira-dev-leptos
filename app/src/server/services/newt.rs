use crate::SERVER_CONFIG;
use crate::error::NewtArticleServiceError;
use crate::server::models::newt_article::{NewtArticle, NewtArticleCollection};
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

    #[allow(dead_code)] // TODO: Newt終了後に削除
    #[instrument]
    async fn fetch_articles(
        &self,
        is_preview: bool,
    ) -> Result<NewtArticleCollection, NewtArticleServiceError> {
        let (base_url, api_token) = if is_preview {
            (&self.newt_base_url, &SERVER_CONFIG.newt_api_token)
        } else {
            (&self.newt_cdn_base_url, &SERVER_CONFIG.newt_cdn_api_token)
        };

        let response = self
            .client
            .get(format!("{base_url}/blog/article"))
            .bearer_auth(api_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NewtArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let articles: NewtArticleCollection = response.json().await?;

        Ok(articles)
    }

    #[allow(dead_code)] // TODO: Newt終了後に削除
    #[instrument]
    pub(crate) async fn fetch_published_articles(
        &self,
    ) -> Result<NewtArticleCollection, NewtArticleServiceError> {
        self.fetch_articles(false).await
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

    #[allow(dead_code)] // TODO: Newt終了後に削除
    #[instrument]
    pub(crate) async fn fetch_published_article<T>(
        &self,
        article_id: T,
    ) -> Result<Option<NewtArticle>, NewtArticleServiceError>
    where
        T: std::fmt::Display + Debug,
    {
        self.fetch_article(article_id, false).await
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

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::SERVER_CONFIG;
    use crate::server::models::newt_article::NewtArticleCollection;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn test_is_previewがfalseの場合はcdn用のトークンとurlを使用してリクエストすること() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock("GET", "/blog/article")
            .match_header(
                "authorization",
                format!("Bearer {}", &SERVER_CONFIG.newt_cdn_api_token).as_str(),
            )
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&NewtArticleCollection::default()).unwrap())
            .create();

        let client = reqwest::Client::new();
        let service = NewtArticleService::new(client, &url, "");

        let result = service.fetch_articles(false).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NewtArticleCollection::default());
    }

    #[tokio::test]
    async fn test_is_previewがtrueの場合はapi用のトークンとurlを使用してリクエストすること() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock("GET", "/blog/article")
            .match_header(
                "authorization",
                format!("Bearer {}", &SERVER_CONFIG.newt_api_token).as_str(),
            )
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&NewtArticleCollection::default()).unwrap())
            .create();

        let client = reqwest::Client::new();
        let service = NewtArticleService::new(client, "", &url);

        let result = service.fetch_articles(true).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NewtArticleCollection::default());
    }

    #[tokio::test]
    async fn test_200_299以外のステータスコードが返された場合はエラーを返すこと() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock("GET", "/blog/article")
            .with_status(500)
            .create();

        let client = reqwest::Client::new();
        let service = NewtArticleService::new(client, &url, "");

        let result = service.fetch_articles(false).await;

        assert!(result.is_err());
        assert_eq!(
            match result.unwrap_err() {
                NewtArticleServiceError::UnexpectedStatusCode(status) => status,
                _ => panic!("Unexpected error type"),
            },
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
