//! cmsクレートのモデルからDTOへの変換

use crate::common::dto::{
    ArticleDetailDto, ArticleMetaDto, ArticlePageDto, ArticleSource, HomePageArticleDto,
};
use crate::common::imgix_url::{extract_base_url, generate_srcset, is_imgix_url};
use crate::common::markdown::{convert_markdown_to_html, sanitize_html};
use crate::constants::{
    COVER_IMAGE_WIDTHS, DATE_DISPLAY_FORMAT, DATE_ISO_FORMAT, HOUR, JST_TZ, THUMBNAIL_NO_IMAGE_URL,
};
use crate::server::utils::url::{
    to_optimize_cover_image_url, to_optimize_og_image_url, to_optimize_thumbnail_url,
};
use blog_romira_dev_cms::{DraftArticleWithCategories, PublishedArticleWithCategories};
use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};
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
            title: article.title,
            thumbnail_url: to_optimize_thumbnail_url(
                article
                    .cover_image_url
                    .as_deref()
                    .unwrap_or(THUMBNAIL_NO_IMAGE_URL),
            ),
            src: format!("/articles/{}", article.slug),
            category: value
                .categories
                .iter()
                .map(|category| category.name.clone())
                .collect(),
            first_published_at: to_jst(article.published_at)
                .format(DATE_DISPLAY_FORMAT)
                .to_string(),
            article_source: ArticleSource::Local,
        }
    }
}

impl From<PublishedArticleWithCategories> for ArticlePageDto {
    #[instrument(skip(value))]
    fn from(value: PublishedArticleWithCategories) -> Self {
        let article = value.article;
        let title = article.title;
        let cover_image_raw = article
            .cover_image_url
            .as_deref()
            .unwrap_or(THUMBNAIL_NO_IMAGE_URL);
        let cover_image_url = to_optimize_cover_image_url(cover_image_raw);
        let cover_image_srcset = if is_imgix_url(cover_image_raw) {
            generate_srcset(extract_base_url(cover_image_raw), &COVER_IMAGE_WIDTHS)
        } else {
            String::new()
        };
        let body = sanitize_html(&convert_markdown_to_html(article.body.as_str()));
        let category: Vec<String> = value
            .categories
            .iter()
            .map(|category| category.name.clone())
            .collect();

        let published_at_jst = to_jst(article.published_at);
        let updated_at_jst = to_jst(article.updated_at);
        let updated_at_rfc3339 = updated_at_jst.to_rfc3339();
        let first_published_at = published_at_jst.format(DATE_DISPLAY_FORMAT).to_string();
        let first_published_at_iso = published_at_jst.format(DATE_ISO_FORMAT).to_string();
        let first_published_at_rfc3339 = published_at_jst.to_rfc3339();

        let id = article.id.to_string();
        let slug = article.slug;
        let description = article.description.unwrap_or_default();
        let og_image_url = to_optimize_og_image_url(
            article
                .cover_image_url
                .as_deref()
                .unwrap_or(THUMBNAIL_NO_IMAGE_URL),
        );

        Self {
            article_detail_dto: ArticleDetailDto {
                title: title.clone(),
                cover_image_url,
                cover_image_srcset,
                body,
                category: category.clone(),
                first_published_at,
                first_published_at_iso,
            },
            article_meta_dto: ArticleMetaDto {
                id,
                slug,
                title,
                description,
                keywords: category,
                og_image_url,
                published_at: updated_at_rfc3339,
                first_published_at: first_published_at_rfc3339,
            },
        }
    }
}

impl From<DraftArticleWithCategories> for ArticlePageDto {
    #[instrument(skip(value))]
    fn from(value: DraftArticleWithCategories) -> Self {
        let article = value.article;
        let title = article.title;
        let cover_image_raw = article
            .cover_image_url
            .as_deref()
            .unwrap_or(THUMBNAIL_NO_IMAGE_URL);
        let cover_image_url = to_optimize_cover_image_url(cover_image_raw);
        let cover_image_srcset = if is_imgix_url(cover_image_raw) {
            generate_srcset(extract_base_url(cover_image_raw), &COVER_IMAGE_WIDTHS)
        } else {
            String::new()
        };
        let body = sanitize_html(&convert_markdown_to_html(article.body.as_str()));
        let category: Vec<String> = value
            .categories
            .iter()
            .map(|category| category.name.clone())
            .collect();

        // プレビュー用なので現在時刻を仮の日付とする（あるいは保存された日時）
        let updated_at_jst = to_jst(article.updated_at);
        let updated_at_rfc3339 = updated_at_jst.to_rfc3339();
        let first_published_at = updated_at_jst.format(DATE_DISPLAY_FORMAT).to_string();
        let first_published_at_iso = updated_at_jst.format(DATE_ISO_FORMAT).to_string();
        let first_published_at_rfc3339 = updated_at_jst.to_rfc3339();

        let id = article.id.to_string();
        let slug = article.slug;
        let description = article.description.unwrap_or_default();
        let og_image_url = to_optimize_og_image_url(
            article
                .cover_image_url
                .as_deref()
                .unwrap_or(THUMBNAIL_NO_IMAGE_URL),
        );

        Self {
            article_detail_dto: ArticleDetailDto {
                title: title.clone(),
                cover_image_url,
                cover_image_srcset,
                body,
                category: category.clone(),
                first_published_at,
                first_published_at_iso,
            },
            article_meta_dto: ArticleMetaDto {
                id,
                slug,
                title,
                description,
                keywords: category,
                og_image_url,
                published_at: updated_at_rfc3339,
                first_published_at: first_published_at_rfc3339,
            },
        }
    }
}
