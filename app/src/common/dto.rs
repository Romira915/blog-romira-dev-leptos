use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageArticleDto {
    pub(crate) title: RwSignal<String>,
    pub(crate) thumbnail_url: RwSignal<String>,
    pub(crate) src: RwSignal<String>,
    pub(crate) category: Vec<RwSignal<String>>,
    pub(crate) first_published_at: RwSignal<String>,
    pub(crate) article_source: ArticleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ArticleSource {
    Newt,
    WordPress,
    Qiita,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageAuthorDto {
    pub(crate) name: RwSignal<String>,
    pub(crate) avatar_url: RwSignal<String>,
    pub(crate) description: RwSignal<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlePageDto {
    pub(crate) article_detail_dto: ArticleDetailDto,
    pub(crate) article_meta_dto: ArticleMetaDto,
}

/// 記事取得の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleResponse {
    Found(ArticlePageDto),
    Redirect(String),
    NotFound(()),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetailDto {
    pub(crate) title: RwSignal<String>,
    pub(crate) cover_image_url: RwSignal<String>,
    pub(crate) body: RwSignal<String>,
    pub(crate) category: Vec<RwSignal<String>>,
    pub(crate) first_published_at: RwSignal<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetaDto {
    pub(crate) id: RwSignal<String>,
    pub(crate) title: RwSignal<String>,
    pub(crate) description: RwSignal<String>,
    pub(crate) keywords: Vec<RwSignal<String>>,
    pub(crate) og_image_url: RwSignal<String>,
    pub(crate) published_at: RwSignal<String>,
    pub(crate) first_published_at: RwSignal<String>,
}
