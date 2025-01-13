use crate::SERVER_CONFIG;
use crate::error::QiitaArticleServiceError;
use crate::server::models::qiita_article::QiitaArticleList;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct QiitaArticleService {
    client: reqwest::Client,
    qiita_base_url: Arc<String>,
}

impl QiitaArticleService {
    pub(crate) fn new(client: reqwest::Client, qiita_base_url: impl ToString) -> Self {
        Self {
            client,
            qiita_base_url: Arc::new(qiita_base_url.to_string()),
        }
    }

    pub(crate) async fn get_articles(&self) -> Result<QiitaArticleList, QiitaArticleServiceError> {
        let (base_url, api_token) = (&self.qiita_base_url, &SERVER_CONFIG.qiita_api_token);

        let response = self
            .client
            .get(format!("{base_url}/api/v2/authenticated_user/items"))
            .bearer_auth(api_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(QiitaArticleServiceError::UnexpectedStatusCode(
                response.status(),
            ));
        }

        let articles: QiitaArticleList = response.json().await?;

        Ok(articles)
    }
}
