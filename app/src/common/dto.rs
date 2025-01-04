use chrono::{DateTime, FixedOffset};

pub(crate) struct HomePageArticleDto {
    pub(crate) title: String,
    pub(crate) thumbnail_url: String,
    pub(crate) category: String,
    pub(crate) published_at: DateTime<FixedOffset>,
}

pub(crate) struct HomePageArticleListDto {
    pub(crate) articles: Vec<HomePageArticleDto>,
}
