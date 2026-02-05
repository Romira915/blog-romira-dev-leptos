use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 画像ライブラリ
/// タイムスタンプはUTCで保存
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub filename: String,
    pub gcs_path: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 公開済み記事
/// タイムスタンプはUTCで保存
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PublishedArticle {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub published_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 下書き記事（新規記事のみ、公開前の下書き）
/// タイムスタンプはUTCで保存
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DraftArticle {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// 公開記事とカテゴリを結合した構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedArticleWithCategories {
    pub article: PublishedArticle,
    pub categories: Vec<Category>,
}

/// 下書き記事とカテゴリを結合した構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftArticleWithCategories {
    pub article: DraftArticle,
    pub categories: Vec<Category>,
}

/// 管理画面用: 記事一覧表示用の統合型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleListItem {
    Published(PublishedArticleWithCategories),
    Draft(DraftArticleWithCategories),
}

impl ArticleListItem {
    pub fn id(&self) -> Uuid {
        match self {
            ArticleListItem::Published(a) => a.article.id,
            ArticleListItem::Draft(a) => a.article.id,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            ArticleListItem::Published(a) => &a.article.title,
            ArticleListItem::Draft(a) => &a.article.title,
        }
    }

    pub fn is_draft(&self) -> bool {
        matches!(self, ArticleListItem::Draft(_))
    }

    pub fn updated_at(&self) -> NaiveDateTime {
        match self {
            ArticleListItem::Published(a) => a.article.updated_at,
            ArticleListItem::Draft(a) => a.article.updated_at,
        }
    }

    pub fn published_at(&self) -> Option<NaiveDateTime> {
        match self {
            ArticleListItem::Published(a) => Some(a.article.published_at),
            ArticleListItem::Draft(_) => None,
        }
    }
}
