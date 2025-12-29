use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmsError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("{0}")]
    ValidationError(String),
}
