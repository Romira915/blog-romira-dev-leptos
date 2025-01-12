use crate::SERVER_CONFIG;
use crate::error::NewtArticleServiceError;
use crate::server::models::newt_article::NewtArticleCollection;
use crate::server::models::newt_author::Author;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct NewtArticleService {
    client: reqwest::Client,
    newt_cdn_base_url: Arc<String>,
    newt_base_url: Arc<String>,
}

impl NewtArticleService {
    pub(crate) fn new(
        client: reqwest::Client,
        newt_cdn_base_url: impl ToString,
        newt_base_url: impl ToString,
    ) -> Self {
        Self {
            client,
            newt_cdn_base_url: Arc::new(newt_cdn_base_url.to_string()),
            newt_base_url: Arc::new(newt_base_url.to_string()),
        }
    }

    async fn get_articles(
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

    pub(crate) async fn get_published_articles(
        &self,
    ) -> Result<NewtArticleCollection, NewtArticleServiceError> {
        self.get_articles(false).await
    }

    pub(crate) async fn get_preview_articles(
        &self,
    ) -> Result<NewtArticleCollection, NewtArticleServiceError> {
        self.get_articles(true).await
    }

    pub(crate) async fn get_author<T>(
        &self,
        author_id: T,
    ) -> Result<Author, NewtArticleServiceError>
    where
        T: std::fmt::Display,
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

        let result = service.get_articles(false).await;

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

        let result = service.get_articles(true).await;

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

        let result = service.get_articles(false).await;

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
