use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewtArticleServiceError {
    #[error("Failed to fetch Newt articles")]
    NewtApiError(#[from] reqwest::Error),
}
