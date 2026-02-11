use crate::error::CmsError;
use crate::models::{ArticleContent, PublishedArticleWithCategories};
use crate::queries::PublishedArticleQuery;
use crate::repositories::PublishedArticleRepository;
use crate::value_objects::{PublishedArticleSlug, PublishedArticleTitle};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use super::utc_now;

/// 公開記事サービス（フロント表示用）
#[derive(Debug, Clone)]
pub struct PublishedArticleService {
    pool: PgPool,
}

impl PublishedArticleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 公開済み記事一覧を取得
    #[instrument(skip(self))]
    pub async fn fetch_all(&self) -> Result<Vec<PublishedArticleWithCategories>, CmsError> {
        PublishedArticleQuery::fetch_all(&self.pool, utc_now()).await
    }

    /// 公開済み記事をIDで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_id(
        &self,
        article_id: Uuid,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        PublishedArticleQuery::fetch_by_id(&self.pool, article_id, utc_now()).await
    }

    /// 公開済み記事をslugで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        PublishedArticleQuery::fetch_by_slug(&self.pool, slug, utc_now()).await
    }

    /// 公開済み記事をIDで取得（管理者用、公開日時フィルタなし）
    #[instrument(skip(self))]
    pub async fn fetch_by_id_for_admin(
        &self,
        article_id: Uuid,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        PublishedArticleQuery::fetch_by_id_for_admin(&self.pool, article_id).await
    }

    /// 公開記事を削除
    #[instrument(skip(self))]
    pub async fn delete(&self, article_id: Uuid) -> Result<(), CmsError> {
        PublishedArticleRepository::delete(&self.pool, article_id).await
    }

    /// 公開記事を更新
    #[instrument(skip(self))]
    pub async fn update(
        &self,
        article_id: Uuid,
        title: &PublishedArticleTitle,
        slug: &PublishedArticleSlug,
        body: &str,
        description: Option<&str>,
        cover_image_url: Option<&str>,
    ) -> Result<(), CmsError> {
        // スラッグ重複チェック（自分自身は除外）
        if PublishedArticleQuery::exists_by_slug(&self.pool, slug.as_str(), Some(article_id))
            .await?
        {
            return Err(CmsError::ValidationError(
                "このスラッグは既に使用されています".to_string(),
            ));
        }

        let content = ArticleContent {
            title: title.as_str(),
            slug: slug.as_str(),
            body,
            description,
            cover_image_url,
        };

        PublishedArticleRepository::update(&self.pool, article_id, &content, utc_now()).await
    }
}
