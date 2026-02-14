use std::sync::Arc;
use tracing::instrument;

#[derive(Debug, thiserror::Error)]
pub enum CloudflarePurgeError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Cloudflare API error (status={status}): {body}")]
    ApiError {
        status: reqwest::StatusCode,
        body: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct CloudflarePurgeService {
    client: reqwest::Client,
    base_url: Arc<String>,
    zone_id: Arc<String>,
    api_token: Arc<String>,
}

impl CloudflarePurgeService {
    pub(crate) fn new(
        client: reqwest::Client,
        zone_id: impl ToString,
        api_token: impl ToString,
    ) -> Self {
        Self {
            client,
            base_url: Arc::new("https://api.cloudflare.com/client/v4".to_string()),
            zone_id: Arc::new(zone_id.to_string()),
            api_token: Arc::new(api_token.to_string()),
        }
    }

    #[cfg(test)]
    fn with_base_url(
        client: reqwest::Client,
        base_url: impl ToString,
        zone_id: impl ToString,
        api_token: impl ToString,
    ) -> Self {
        Self {
            client,
            base_url: Arc::new(base_url.to_string()),
            zone_id: Arc::new(zone_id.to_string()),
            api_token: Arc::new(api_token.to_string()),
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn purge_tags(&self, tags: &[String]) -> Result<(), CloudflarePurgeError> {
        if tags.is_empty() {
            return Ok(());
        }

        let url = format!("{}/zones/{}/purge_cache", self.base_url, self.zone_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(self.api_token.as_str())
            .json(&serde_json::json!({ "tags": tags }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CloudflarePurgeError::ApiError { status, body });
        }

        Ok(())
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn purge_tagsで正常にパージできること() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/zones/test-zone/purge_cache")
            .match_header("authorization", "Bearer test-token")
            .match_body(mockito::Matcher::Json(
                serde_json::json!({"tags": ["top-page", "article:my-slug"]}),
            ))
            .with_status(200)
            .with_body(r#"{"success":true}"#)
            .create_async()
            .await;

        let service = CloudflarePurgeService::with_base_url(
            reqwest::Client::new(),
            server.url(),
            "test-zone",
            "test-token",
        );

        let result = service
            .purge_tags(&["top-page".to_string(), "article:my-slug".to_string()])
            .await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn purge_tagsでapiエラー時にエラーを返すこと() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/zones/test-zone/purge_cache")
            .with_status(403)
            .with_body(r#"{"success":false,"errors":[{"message":"Forbidden"}]}"#)
            .create_async()
            .await;

        let service = CloudflarePurgeService::with_base_url(
            reqwest::Client::new(),
            server.url(),
            "test-zone",
            "test-token",
        );

        let result = service.purge_tags(&["top-page".to_string()]).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, CloudflarePurgeError::ApiError { status, .. } if status == reqwest::StatusCode::FORBIDDEN),
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn purge_tagsで空タグの場合はapiを呼ばないこと() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/zones/test-zone/purge_cache")
            .expect(0)
            .create_async()
            .await;

        let service = CloudflarePurgeService::with_base_url(
            reqwest::Client::new(),
            server.url(),
            "test-zone",
            "test-token",
        );

        let result = service.purge_tags(&[]).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }
}
