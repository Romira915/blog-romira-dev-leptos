use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NewtArticleCollection {
    pub(crate) skip: u32,
    pub(crate) limit: u32,
    pub(crate) total: u32,
    pub(crate) items: Vec<NewtArticle>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
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
    #[serde(rename = "profileImage")]
    pub(crate) profile_image_id: Option<String>,
    pub(crate) biography: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Meta {
    pub(crate) title: String,
    pub(crate) description: String,
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
