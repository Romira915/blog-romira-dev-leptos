use crate::common::dto::{ArticleSource, HomePageArticleDto};
use crate::constants::{DATE_DISPLAY_FORMAT, HOUR, JST_TZ};
use chrono::{DateTime, FixedOffset, Utc};
use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub(crate) type QiitaArticleList = Vec<QiitaArticle>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QiitaArticle {
    pub rendered_body: String,
    pub body: String,
    pub coediting: bool,
    pub comments_count: i64,
    pub created_at: DateTime<Utc>,
    pub group: Option<Group>,
    pub id: String,
    pub likes_count: i64,
    pub private: bool,
    pub reactions_count: i64,
    pub stocks_count: i64,
    pub tags: Vec<Tag>,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub user: User,
    pub page_views_count: Option<i64>,
    pub team_membership: Option<TeamMembership>,
    pub organization_url_name: Option<String>,
    pub slide: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub og_image_url: String,
}

impl From<QiitaArticle> for HomePageArticleDto {
    #[instrument]
    fn from(value: QiitaArticle) -> Self {
        Self {
            title: RwSignal::new(value.title),
            thumbnail_url: RwSignal::new(value.og_image_url),
            src: RwSignal::new(value.url),
            category: value
                .tags
                .iter()
                .map(|tag| RwSignal::new(tag.name.clone()))
                .collect(),
            published_at: RwSignal::new(
                value
                    .created_at
                    .with_timezone(&FixedOffset::east_opt(JST_TZ * HOUR).unwrap())
                    .format(DATE_DISPLAY_FORMAT)
                    .to_string(),
            ),
            article_source: ArticleSource::Qiita,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub created_at: String,
    pub description: String,
    pub name: String,
    pub private: bool,
    pub updated_at: String,
    pub url_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub versions: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub description: Option<String>,
    pub facebook_id: Option<String>,
    pub followees_count: i64,
    pub followers_count: i64,
    pub github_login_name: Option<String>,
    pub id: String,
    pub items_count: i64,
    pub linkedin_id: Option<String>,
    pub location: Option<String>,
    pub name: Option<String>,
    pub organization: Option<String>,
    pub permanent_id: i64,
    pub profile_image_url: String,
    pub team_only: bool,
    pub twitter_screen_name: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamMembership {
    pub name: String,
}
