use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub draft: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

/// 記事とカテゴリを結合した構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleWithCategories {
    pub article: Article,
    pub categories: Vec<Category>,
}
