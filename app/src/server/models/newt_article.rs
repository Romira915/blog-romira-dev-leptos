use crate::common::dto::{
    ArticleDetailDto, ArticleMetaDto, ArticlePageDto, ArticleSource, HomePageArticleDto,
};
use crate::constants::{DATE_DISPLAY_FORMAT, HOUR, JST_TZ, THUMBNAIL_NO_IMAGE_URL};
use crate::server::utils::url::to_optimize_thumbnail_url;
use chrono::{DateTime, FixedOffset, Utc};
use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewtArticleCollection {
    pub(crate) skip: u32,
    pub(crate) limit: u32,
    pub(crate) total: u32,
    pub(crate) items: Vec<NewtArticle>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewtArticle {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    #[serde(rename = "_sys")]
    pub(crate) sys: Sys,
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) meta: Option<Meta>,
    pub(crate) body: Option<String>,
    pub(crate) cover_image: Option<Image>,
    pub(crate) author: Option<AuthorInArticle>,
    pub(crate) categories: Option<Vec<Category>>,
}

impl From<NewtArticle> for HomePageArticleDto {
    #[instrument]
    fn from(value: NewtArticle) -> Self {
        Self {
            title: RwSignal::new(value.title),
            thumbnail_url: RwSignal::new(to_optimize_thumbnail_url(
                value.cover_image.as_ref().map_or_else(
                    || THUMBNAIL_NO_IMAGE_URL,
                    |cover_image| cover_image.src.as_str(),
                ),
            )),
            src: RwSignal::new(format!("/articles/{}", value.id)),
            category: value
                .categories
                .as_ref()
                .map_or_else(Vec::new, |categories| {
                    categories
                        .iter()
                        .map(|category| RwSignal::new(category.name.clone()))
                        .collect()
                }),
            first_published_at: RwSignal::new(
                value
                    .sys
                    .raw
                    .first_published_at
                    .unwrap_or(DateTime::from_timestamp(0, 0).unwrap())
                    .with_timezone(&FixedOffset::east_opt(JST_TZ * HOUR).unwrap())
                    .format(DATE_DISPLAY_FORMAT)
                    .to_string(),
            ),
            article_source: ArticleSource::Newt,
        }
    }
}

impl From<NewtArticle> for ArticlePageDto {
    #[instrument]
    fn from(value: NewtArticle) -> Self {
        let title = RwSignal::new(value.title);
        let cover_image_url = RwSignal::new(to_optimize_thumbnail_url(
            value.cover_image.as_ref().map_or_else(
                || THUMBNAIL_NO_IMAGE_URL,
                |cover_image| cover_image.src.as_str(),
            ),
        ));
        let body = RwSignal::new(value.body.unwrap_or_default());
        let category = value
            .categories
            .as_ref()
            .map_or_else(Vec::new, |categories| {
                categories
                    .iter()
                    .map(|category| RwSignal::new(category.name.clone()))
                    .collect()
            });
        let published_at_rfc3339 = RwSignal::new(
            value
                .sys
                .raw
                .published_at
                .unwrap_or(DateTime::from_timestamp(0, 0).unwrap())
                .with_timezone(&FixedOffset::east_opt(JST_TZ * HOUR).unwrap())
                .to_rfc3339(),
        );
        let first_published_at_date_time = value
            .sys
            .raw
            .first_published_at
            .unwrap_or(DateTime::from_timestamp(0, 0).unwrap())
            .with_timezone(&FixedOffset::east_opt(JST_TZ * HOUR).unwrap());
        let first_published_at = RwSignal::new(
            first_published_at_date_time
                .format(DATE_DISPLAY_FORMAT)
                .to_string(),
        );
        let first_published_at_rfc3339 = RwSignal::new(first_published_at_date_time.to_rfc3339());
        let id = RwSignal::new(value.id);
        let description = RwSignal::new(
            value
                .meta
                .as_ref()
                .map_or_else(|| "".to_string(), |meta| meta.description.clone()),
        );
        let og_image_url =
            RwSignal::new(to_optimize_thumbnail_url(value.meta.as_ref().map_or_else(
                || THUMBNAIL_NO_IMAGE_URL,
                |meta| {
                    meta.og_image
                        .as_ref()
                        .map_or_else(|| THUMBNAIL_NO_IMAGE_URL, |og_image| og_image.src.as_str())
                },
            )));

        Self {
            article_detail_dto: ArticleDetailDto {
                title,
                cover_image_url,
                body,
                category: category.clone(),
                first_published_at,
            },
            article_meta_dto: ArticleMetaDto {
                id,
                title,
                description,
                keywords: category,
                og_image_url,
                published_at: published_at_rfc3339,
                first_published_at: first_published_at_rfc3339,
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Category {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    #[serde(rename = "_sys")]
    pub(crate) sys: Sys,
    pub(crate) name: String,
    pub(crate) slug: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorInArticle {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    #[serde(rename = "_sys")]
    pub(crate) sys: Sys,
    pub(crate) full_name: String,
    pub(crate) profile_image_id: Option<String>,
    pub(crate) biography: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Meta {
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(rename = "ogImage")]
    pub(crate) og_image: Option<Image>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    pub(crate) src: String,
    pub(crate) file_type: String,
    pub(crate) file_size: u32,
    pub(crate) file_name: String,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Sys {
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) raw: Raw,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Raw {
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) first_published_at: Option<DateTime<Utc>>,
    pub(crate) published_at: Option<DateTime<Utc>>,
}
