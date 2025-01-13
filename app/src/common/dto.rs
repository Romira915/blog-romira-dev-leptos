use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageArticleDto {
    pub(crate) title: RwSignal<String>,
    pub(crate) thumbnail_url: RwSignal<String>,
    pub(crate) src: RwSignal<String>,
    pub(crate) category: Vec<RwSignal<String>>,
    pub(crate) published_at: RwSignal<String>,
    pub(crate) article_source: ArticleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ArticleSource {
    Newt,
    WordPress,
    Qiita,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageAuthorDto {
    pub(crate) name: RwSignal<String>,
    pub(crate) avatar_url: RwSignal<String>,
    pub(crate) description: RwSignal<String>,
}
