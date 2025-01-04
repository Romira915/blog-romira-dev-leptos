use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct Category {
    pub(crate) id: u64,
    pub(crate) count: u64,
    pub(crate) description: String,
    pub(crate) link: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) taxonomy: String,
    pub(crate) parent: u64,
    pub(crate) meta: Vec<String>,
    pub(crate) _links: crate::server::models::word_press_article::Links,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct Links {
    #[serde(rename = "self")]
    pub(crate) self_: Vec<crate::server::models::word_press_article::Link>,
    pub(crate) collection: Vec<crate::server::models::word_press_article::Link>,
    pub(crate) about: Vec<crate::server::models::word_press_article::Link>,
    #[serde(rename = "wp:post_type")]
    pub(crate) wp_post_type: Vec<crate::server::models::word_press_article::Link>,
    pub(crate) curies: Vec<Cury>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct Link {
    pub(crate) href: String,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub(crate) struct Cury {
    pub(crate) name: String,
    pub(crate) href: String,
    pub(crate) templated: bool,
}
