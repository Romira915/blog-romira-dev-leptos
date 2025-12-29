use crate::error::CmsError;
use crate::models::DraftArticleWithCategories;
use crate::queries::{DraftArticleQuery, PublishedArticleQuery};
use crate::repositories::{DraftArticleRepository, PublishedArticleRepository};
use crate::value_objects::PublishedArticleSlug;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use super::utc_now;

/// 下書き記事サービス（管理画面用）
#[derive(Debug, Clone)]
pub struct DraftArticleService {
    pool: PgPool,
}

impl DraftArticleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 下書き記事一覧を取得
    #[instrument(skip(self))]
    pub async fn fetch_all(&self) -> Result<Vec<DraftArticleWithCategories>, CmsError> {
        DraftArticleQuery::fetch_all(&self.pool).await
    }

    /// 下書き記事をIDで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_id(
        &self,
        article_id: Uuid,
    ) -> Result<Option<DraftArticleWithCategories>, CmsError> {
        DraftArticleQuery::fetch_by_id(&self.pool, article_id).await
    }

    /// 下書き記事を作成
    #[instrument(skip(self))]
    pub async fn create(
        &self,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<Uuid, CmsError> {
        DraftArticleRepository::create(&self.pool, title, slug, body, description, utc_now()).await
    }

    /// 下書き記事を更新
    #[instrument(skip(self))]
    pub async fn update(
        &self,
        article_id: Uuid,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<(), CmsError> {
        DraftArticleRepository::update(
            &self.pool,
            article_id,
            title,
            slug,
            body,
            description,
            utc_now(),
        )
        .await
    }

    /// 下書き記事を保存（Upsert: 存在しなければ作成、存在すれば更新）
    #[instrument(skip(self))]
    pub async fn save(
        &self,
        article_id: Uuid,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<(), CmsError> {
        DraftArticleRepository::upsert(
            &self.pool,
            article_id,
            title,
            slug,
            body,
            description,
            utc_now(),
        )
        .await
    }

    /// 下書きを公開（draft_articles → published_articles に移動）
    #[instrument(skip(self))]
    pub async fn publish(&self, draft_id: Uuid) -> Result<Uuid, CmsError> {
        let draft = DraftArticleQuery::fetch_by_id(&self.pool, draft_id)
            .await?
            .ok_or(CmsError::NotFound)?;

        // スラッグのバリデーション
        let slug = PublishedArticleSlug::new(draft.article.slug.clone())?;

        // スラッグ重複チェック（新規公開なので除外IDなし）
        if PublishedArticleQuery::exists_by_slug(&self.pool, slug.as_str(), None).await? {
            return Err(CmsError::ValidationError(
                "このスラッグは既に使用されています".to_string(),
            ));
        }

        let now = utc_now();

        // 公開記事を作成
        let published_id =
            PublishedArticleRepository::create_from_draft(&self.pool, &draft, now).await?;

        // 下書きを削除
        DraftArticleRepository::delete(&self.pool, draft_id).await?;

        Ok(published_id)
    }

    /// 下書き記事を削除
    #[instrument(skip(self))]
    pub async fn delete(&self, article_id: Uuid) -> Result<(), CmsError> {
        DraftArticleRepository::delete(&self.pool, article_id).await
    }
}
