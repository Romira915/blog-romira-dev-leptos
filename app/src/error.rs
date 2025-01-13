use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum NewtArticleServiceError {
    #[error(transparent)]
    FailedReqwestSend(#[from] reqwest::Error),
    #[error("Failed to api response status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

#[derive(Error, Debug)]
pub(crate) enum WordPressArticleServiceError {
    #[error(transparent)]
    FailedReqwestSend(#[from] reqwest::Error),
    #[error("Failed to api response status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

#[derive(Error, Debug)]
pub(crate) enum QiitaArticleServiceError {
    #[error(transparent)]
    FailedReqwestSend(#[from] reqwest::Error),
    #[error("Failed to api response status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum GetArticlesError {
    #[error("Failed to get articles from NewtArticleService: {0}")]
    NewtArticleServiceGetArticles(String),
    #[error("Failed to get author from NewtArticleService: {0}")]
    NewtArticleServiceGetAuthor(String),
    #[error("Failed to get articles from WordPressArticleService: {0}")]
    WordPressArticleService(String),
    #[error("Failed to get articles from QiitaArticleService: {0}")]
    QiitaArticleService(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl FromStr for GetArticlesError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Unknown(s.to_string()))
    }
}
