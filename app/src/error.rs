use thiserror::Error;

#[derive(Error, Debug)]
pub enum HomePageError {
    #[error("Failed to fetch Newt articles")]
    NewtApiError {
        #[from]
        #[source]
        source: reqwest::Error,
    },
}
