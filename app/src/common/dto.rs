use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HomePageArticleDto {
    pub(crate) title: String,
    pub(crate) thumbnail_url: String,
    pub(crate) src: String,
    pub(crate) category: String,
    pub(crate) published_at: DateTime<FixedOffset>,
}
