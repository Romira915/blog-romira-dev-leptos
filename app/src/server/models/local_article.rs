//! cmsクレートのモデルからDTOへの変換

use crate::common::dto::{
    ArticleDetailDto, ArticleMetaDto, ArticlePageDto, ArticleSource, HomePageArticleDto,
};
use crate::common::imgix_url::{extract_base_url, generate_srcset, is_imgix_url};
use crate::common::markdown::convert_markdown_to_html;
use crate::constants::{
    COVER_IMAGE_WIDTHS, DATE_DISPLAY_FORMAT, HOUR, JST_TZ, THUMBNAIL_NO_IMAGE_URL,
};
use crate::server::utils::url::{
    to_optimize_cover_image_url, to_optimize_og_image_url, to_optimize_thumbnail_url,
};
use blog_romira_dev_cms::PublishedArticleWithCategories;
use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};
use leptos::prelude::RwSignal;
use tracing::instrument;

/// NaiveDateTime (UTC) をJSTのDateTimeに変換
fn to_jst(naive: NaiveDateTime) -> chrono::DateTime<FixedOffset> {
    let jst = FixedOffset::east_opt(JST_TZ * HOUR).unwrap();
    Utc.from_utc_datetime(&naive).with_timezone(&jst)
}

impl From<PublishedArticleWithCategories> for HomePageArticleDto {
    #[instrument(skip(value))]
    fn from(value: PublishedArticleWithCategories) -> Self {
        let article = value.article;
        Self {
            title: RwSignal::new(article.title),
            thumbnail_url: RwSignal::new(to_optimize_thumbnail_url(
                article
                    .cover_image_url
                    .as_deref()
                    .unwrap_or(THUMBNAIL_NO_IMAGE_URL),
            )),
            src: RwSignal::new(format!("/articles/{}", article.slug)),
            category: value
                .categories
                .iter()
                .map(|category| RwSignal::new(category.name.clone()))
                .collect(),
            first_published_at: RwSignal::new(
                to_jst(article.published_at)
                    .format(DATE_DISPLAY_FORMAT)
                    .to_string(),
            ),
            article_source: ArticleSource::Local,
        }
    }
}

impl From<PublishedArticleWithCategories> for ArticlePageDto {
    #[instrument(skip(value))]
    fn from(value: PublishedArticleWithCategories) -> Self {
        let article = value.article;
        let title = RwSignal::new(article.title);
        let cover_image_raw = article
            .cover_image_url
            .as_deref()
            .unwrap_or(THUMBNAIL_NO_IMAGE_URL);
        let cover_image_url = RwSignal::new(to_optimize_cover_image_url(cover_image_raw));
        let cover_image_srcset = RwSignal::new(if is_imgix_url(cover_image_raw) {
            generate_srcset(extract_base_url(cover_image_raw), &COVER_IMAGE_WIDTHS)
        } else {
            String::new()
        });
        let body = RwSignal::new(convert_markdown_to_html(article.body.as_str()));
        let category: Vec<RwSignal<String>> = value
            .categories
            .iter()
            .map(|category| RwSignal::new(category.name.clone()))
            .collect();

        let published_at_jst = to_jst(article.published_at);
        let published_at_rfc3339 = RwSignal::new(published_at_jst.to_rfc3339());
        let first_published_at =
            RwSignal::new(published_at_jst.format(DATE_DISPLAY_FORMAT).to_string());
        let first_published_at_rfc3339 = RwSignal::new(published_at_jst.to_rfc3339());

        let id = RwSignal::new(article.id.to_string());
        let description = RwSignal::new(article.description.unwrap_or_default());
        let og_image_url = RwSignal::new(to_optimize_og_image_url(
            article
                .cover_image_url
                .as_deref()
                .unwrap_or(THUMBNAIL_NO_IMAGE_URL),
        ));

        Self {
            article_detail_dto: ArticleDetailDto {
                title,
                cover_image_url,
                cover_image_srcset,
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
