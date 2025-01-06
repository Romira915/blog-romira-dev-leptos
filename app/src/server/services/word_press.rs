use crate::constants::PRTIMES_WORD_PRESS_AUTHOR_ID;
use crate::error::WordPressArticleServiceError;
use crate::server::models::word_press_article::WordPressArticle;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct WordPressArticleService {
    client: reqwest::Client,
    base_url: Arc<String>,
}

impl WordPressArticleService {
    pub(crate) fn new(client: reqwest::Client, base_url: impl ToString) -> Self {
        Self {
            client,
            base_url: Arc::new(base_url.to_string()),
        }
    }

    pub(crate) async fn get_articles(
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

        let articles: Vec<WordPressArticle> = response.json().await?;

        Ok(articles)
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
    async fn test_200_299以外のステータスコードが返された場合はエラーを返すこと() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        server
            .mock("GET", format!("/wp-json/wp/v2/posts?author={}", PRTIMES_WORD_PRESS_AUTHOR_ID).as_str())
            .with_status(500)
            .create();

        let client = reqwest::Client::new();
        let service = WordPressArticleService::new(client, &url);

        let result = service.get_articles().await;

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
