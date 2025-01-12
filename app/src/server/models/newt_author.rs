use crate::common::dto::HomePageAuthorDto;
use crate::server::models::newt_article::{Image, Sys};
use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Author {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    #[serde(rename = "_sys")]
    pub(crate) sys: Sys,
    pub(crate) full_name: String,
    pub(crate) profile_image: Option<Image>,
    pub(crate) biography: Option<String>,
}

impl From<Author> for HomePageAuthorDto {
    fn from(value: Author) -> Self {
        Self {
            name: RwSignal::new(value.full_name),
            avatar_url: value
                .profile_image
                .map(|image| RwSignal::new(image.src))
                .unwrap_or_default(),
            description: value
                .biography
                .map(|biography| RwSignal::new(biography))
                .unwrap_or_default(),
        }
    }
}
