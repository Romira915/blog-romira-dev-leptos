use crate::common::dto::{ArticleSource, HomePageArticleDto};
use crate::constants::{DATE_DISPLAY_FORMAT, HOUR, JST_TZ};
use crate::server::models::word_press_category::Category;
use chrono::{FixedOffset, NaiveDateTime, TimeZone};
use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct WordPressArticle {
    #[serde(rename = "id")]
    pub(crate) id: u64,
    pub(crate) date: NaiveDateTime,
    pub(crate) date_gmt: NaiveDateTime,
    pub(crate) guid: Guid,
    pub(crate) modified: String,
    pub(crate) modified_gmt: String,
    pub(crate) slug: String,
    pub(crate) status: String,
    pub(crate) r#type: String,
    pub(crate) link: String,
    pub(crate) title: Title,
    pub(crate) content: Content,
    pub(crate) excerpt: Excerpt,
    pub(crate) author: u64,
    pub(crate) featured_media: u64,
    pub(crate) comment_status: String,
    pub(crate) ping_status: String,
    pub(crate) sticky: bool,
    pub(crate) template: Option<String>,
    pub(crate) format: String,
    pub(crate) meta: Meta,
    pub(crate) categories: Vec<u64>,
    pub(crate) tags: Vec<u64>,
    pub(crate) jetpack_publicize_connections: Vec<String>,
    pub(crate) jetpack_featured_media_url: String,
    pub(crate) jetpack_likes_enabled: bool,
    pub(crate) jetpack_sharing_enabled: bool,
    pub(crate) jetpack_shortlink: String,
    #[serde(rename = "_links")]
    pub(crate) links: Links,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) category_names: Vec<Category>,
}

impl From<WordPressArticle> for HomePageArticleDto {
    fn from(value: WordPressArticle) -> Self {
        Self {
            title: RwSignal::new(value.title.rendered),
            thumbnail_url: RwSignal::new(value.jetpack_featured_media_url),
            src: RwSignal::new(value.link),
            category: value
                .category_names
                .iter()
                .map(|category| RwSignal::new(category.name.clone()))
                .collect(),
            published_at: RwSignal::new(
                FixedOffset::east(JST_TZ * HOUR)
                    .from_utc_datetime(&value.date)
                    .format(DATE_DISPLAY_FORMAT)
                    .to_string(),
            ),
            article_source: ArticleSource::WordPress,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Guid {
    pub(crate) rendered: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Title {
    pub(crate) rendered: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Content {
    pub(crate) rendered: String,
    pub(crate) protected: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Excerpt {
    pub(crate) rendered: String,
    pub(crate) protected: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Meta {
    #[serde(rename = "_coblocks_attr")]
    pub(crate) coblocks_attr: String,
    #[serde(rename = "_coblocks_dimensions")]
    pub(crate) coblocks_dimensions: String,
    #[serde(rename = "_coblocks_responsive_height")]
    pub(crate) coblocks_responsive_height: String,
    #[serde(rename = "_coblocks_accordion_ie_support")]
    pub(crate) coblocks_accordion_ie_support: String,
    pub(crate) advanced_seo_description: String,
    pub(crate) jetpack_seo_html_title: String,
    pub(crate) jetpack_seo_noindex: bool,
    pub(crate) swell_btn_cv_data: String,
    pub(crate) jetpack_publicize_message: String,
    pub(crate) jetpack_publicize_feature_enabled: bool,
    pub(crate) jetpack_social_post_already_shared: bool,
    pub(crate) jetpack_social_options: JetpackSocialOptions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct JetpackSocialOptions {
    pub(crate) image_generator_settings: ImageGeneratorSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct ImageGeneratorSettings {
    pub(crate) template: String,
    pub(crate) enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Links {
    #[serde(rename = "self")]
    pub(crate) self_: Vec<Link>,
    pub(crate) collection: Vec<Link>,
    pub(crate) about: Vec<Link>,
    pub(crate) author: Vec<Link>,
    pub(crate) replies: Vec<Link>,
    #[serde(rename = "version-history")]
    pub(crate) version_history: Vec<Link>,
    #[serde(rename = "predecessor-version")]
    pub(crate) predecessor_version: Vec<Link>,
    #[serde(rename = "wp:featuredmedia")]
    pub(crate) wp_featuredmedia: Vec<Link>,
    #[serde(rename = "wp:attachment")]
    pub(crate) wp_attachment: Vec<Link>,
    #[serde(rename = "wp:term")]
    pub(crate) wp_term: Vec<TermLink>,
    pub(crate) curies: Vec<CuriLink>,
}

/// NOTE: 型定義がめんどくさいのでスキーマを省略している．
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Link {
    pub(crate) href: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct TermLink {
    pub(crate) taxonomy: String,
    pub(crate) embeddable: bool,
    pub(crate) href: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct CuriLink {
    pub(crate) name: String,
    pub(crate) href: String,
    pub(crate) templated: bool,
}
