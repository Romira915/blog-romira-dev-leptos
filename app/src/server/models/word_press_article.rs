use crate::common::dto::{ArticleSource, HomePageArticleDto};
use crate::constants::{DATE_DISPLAY_FORMAT, HOUR, JST_TZ};
use crate::server::models::word_press_category::Category;
use chrono::{FixedOffset, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct WordPressArticle {
    pub(crate) date: NaiveDateTime,
    pub(crate) link: String,
    pub(crate) title: Title,
    pub(crate) categories: Vec<u64>,
    pub(crate) jetpack_featured_media_url: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) category_names: Vec<Category>,
}

impl From<WordPressArticle> for HomePageArticleDto {
    #[instrument]
    fn from(value: WordPressArticle) -> Self {
        Self {
            title: value.title.rendered,
            thumbnail_url: value.jetpack_featured_media_url,
            src: value.link,
            category: value
                .category_names
                .iter()
                .map(|category| category.name.clone())
                .collect(),
            first_published_at: FixedOffset::east_opt(JST_TZ * HOUR)
                .unwrap()
                .from_utc_datetime(&value.date)
                .format(DATE_DISPLAY_FORMAT)
                .to_string(),
            article_source: ArticleSource::WordPress,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Title {
    pub(crate) rendered: String,
}
