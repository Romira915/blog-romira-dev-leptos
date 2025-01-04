use crate::SERVER_CONFIG;
use crate::error::NewtArticleServiceError;
use crate::server::models::newt_article::NewtArticleCollection;

pub(crate) struct NewtArticleService<'a> {
    client: reqwest::Client,
    newt_cdn_base_url: &'a str,
    newt_base_url: &'a str,
}

impl<'a> NewtArticleService<'a> {
    pub(crate) fn new(
        client: reqwest::Client,
        newt_cdn_base_url: &'a str,
        newt_base_url: &'a str,
    ) -> Self {
        Self {
            client,
            newt_cdn_base_url,
            newt_base_url,
        }
    }

    pub(crate) async fn get_newt_articles(
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

        let articles: NewtArticleCollection = response.json().await?;

        Ok(articles)
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use crate::SERVER_CONFIG;
    use crate::server::models::newt_article::NewtArticleCollection;
    use crate::server::services::NewtArticleService;

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

        let result = service.get_newt_articles(false).await;

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

        let result = service.get_newt_articles(true).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), NewtArticleCollection::default());
    }
}
