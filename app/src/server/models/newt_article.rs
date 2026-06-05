use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
