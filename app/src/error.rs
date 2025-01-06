use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum NewtArticleServiceError {
    #[error(transparent)]
    ReqwestSendError(#[from] reqwest::Error),
    #[error("Failed to api response status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

#[derive(Error, Debug)]
pub(crate) enum WordPressArticleServiceError {
    #[error(transparent)]
    ReqwestSendError(#[from] reqwest::Error),
    #[error("Failed to api response status code: {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}
