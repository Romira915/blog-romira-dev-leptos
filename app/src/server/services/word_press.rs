use crate::constants::PRTIMES_WORD_PRESS_AUTHOR_ID;
use crate::error::WordPressArticleServiceError;
use crate::server::models::word_press_article::WordPressArticle;
use crate::server::models::word_press_category::Category;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, Clone)]
pub(crate) struct WordPressArticleService {
    client: reqwest::Client,
    base_url: Arc<String>,
}

impl WordPressArticleService {
    #[instrument]
    pub(crate) fn new(client: reqwest::Client, base_url: impl ToString + Debug) -> Self {
        Self {
            client,
            base_url: Arc::new(base_url.to_string()),
        }
    }

    #[instrument]
    pub(crate) async fn fetch_articles(
        &self,
    ) -> Result<Vec<WordPressArticle>, WordPressArticleServiceError> {
        let response = self
            .client
            .get(format!(
                "{}/wp-json/wp/v2/posts?author={}",
                self.base_url, PRTIMES_WORD_PRESS_AUTHOR_ID
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(WordPressArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let mut articles: Vec<WordPressArticle> = response.json().await?;
        for article in &mut articles {
            let mut category = vec![];
            for id in article.categories.iter() {
                category.push(self.fetch_categories(*id).await?);
            }

            article.category_names = category;
        }

        Ok(articles)
    }

    #[instrument]
    async fn fetch_categories(&self, id: u64) -> Result<Category, WordPressArticleServiceError> {
        let response = self
            .client
            .get(format!("{}/wp-json/wp/v2/categories/{}", self.base_url, id))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(WordPressArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let categories: Category = response.json().await?;

        Ok(categories)
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use crate::constants::PRTIMES_WORD_PRESS_AUTHOR_ID;
    use crate::error::WordPressArticleServiceError;
    use crate::server::services::word_press::WordPressArticleService;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_正常系() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock(
                "GET",
                format!("/wp-json/wp/v2/posts?author={PRTIMES_WORD_PRESS_AUTHOR_ID}").as_str(),
            )
            .with_body(r#"[]"#)
            .create();

        let client = reqwest::Client::new();
        let service = WordPressArticleService::new(client, &url);

        let result = service.fetch_articles().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn test_response_jsonのスキーマが異なる場合エラーを返すこと() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock(
                "GET",
                format!("/wp-json/wp/v2/posts?author={PRTIMES_WORD_PRESS_AUTHOR_ID}").as_str(),
            )
            .with_body(r#"{"id": 1}"#)
            .create();

        let client = reqwest::Client::new();
        let service = WordPressArticleService::new(client, &url);

        let result = service.fetch_articles().await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WordPressArticleServiceError::FailedReqwestSend(_)
        ));
    }

    #[tokio::test]
    async fn test_200_299以外のステータスコードが返された場合はエラーを返すこと() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock(
                "GET",
                format!("/wp-json/wp/v2/posts?author={PRTIMES_WORD_PRESS_AUTHOR_ID}").as_str(),
            )
            .with_status(500)
            .create();

        let client = reqwest::Client::new();
        let service = WordPressArticleService::new(client, &url);

        let result = service.fetch_articles().await;

        assert!(result.is_err());
        assert_eq!(
            match result.unwrap_err() {
                WordPressArticleServiceError::UnexpectedStatusCode(status) => status,
                _ => panic!("Unexpected error type"),
            },
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
