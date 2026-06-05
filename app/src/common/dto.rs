use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageArticleDto {
    pub(crate) title: String,
    pub(crate) thumbnail_url: String,
    pub(crate) src: String,
    pub(crate) category: Vec<String>,
    pub(crate) first_published_at: String,
    pub(crate) article_source: ArticleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ArticleSource {
    WordPress,
    Qiita,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomePageAuthorDto {
    pub(crate) name: String,
    pub(crate) avatar_url: String,
    pub(crate) description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlePageDto {
    pub(crate) article_detail_dto: ArticleDetailDto,
    pub(crate) article_meta_dto: ArticleMetaDto,
}

/// 記事取得の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleResponse {
    Found(Box<ArticlePageDto>),
    Redirect(String),
    NotFound(()),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetailDto {
    pub(crate) title: String,
    pub(crate) cover_image_url: String,
    pub(crate) cover_image_srcset: String,
    pub(crate) body: String,
    pub(crate) category: Vec<String>,
    pub(crate) first_published_at: String,
    pub(crate) first_published_at_iso: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetaDto {
    pub(crate) id: String,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) keywords: Vec<String>,
    pub(crate) og_image_url: String,
    pub(crate) published_at: String,
    pub(crate) first_published_at: String,
}
