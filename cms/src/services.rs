use crate::error::CmsError;
use crate::models::{ArticleListItem, DraftArticleWithCategories, PublishedArticleWithCategories};
use crate::queries::{AdminArticleQuery, DraftArticleQuery, PublishedArticleQuery};
use crate::repositories::{DraftArticleRepository, PublishedArticleRepository};
use crate::value_objects::{PublishedArticleSlug, PublishedArticleTitle};
use chrono::{NaiveDateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 現在時刻をUTC NaiveDateTimeで取得
fn utc_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

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
    #[instrument(skip(pool))]
    pub async fn fetch_by_id_for_admin(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        PublishedArticleQuery::fetch_by_id_for_admin(pool, article_id).await
    }

    /// 公開記事を更新
    #[instrument(skip(pool))]
    pub async fn update(
        pool: &PgPool,
        article_id: Uuid,
        title: &PublishedArticleTitle,
        slug: &PublishedArticleSlug,
        body: &str,
        description: Option<&str>,
    ) -> Result<(), CmsError> {
        // スラッグ重複チェック（自分自身は除外）
        if PublishedArticleQuery::exists_by_slug(pool, slug.as_str(), Some(article_id)).await? {
            return Err(CmsError::ValidationError(
                "このスラッグは既に使用されています".to_string(),
            ));
        }

        PublishedArticleRepository::update(
            pool,
            article_id,
            title.as_str(),
            slug.as_str(),
            body,
            description,
            utc_now(),
        )
        .await
    }
}

/// 下書き記事サービス（管理画面用）
#[derive(Debug, Clone)]
pub struct DraftArticleService;

impl DraftArticleService {
    /// 下書き記事一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<DraftArticleWithCategories>, CmsError> {
        DraftArticleQuery::fetch_all(pool).await
    }

    /// 下書き記事をIDで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_id(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Option<DraftArticleWithCategories>, CmsError> {
        DraftArticleQuery::fetch_by_id(pool, article_id).await
    }

    /// 下書き記事を作成
    #[instrument(skip(pool))]
    pub async fn create(
        pool: &PgPool,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<Uuid, CmsError> {
        DraftArticleRepository::create(pool, title, slug, body, description, utc_now()).await
    }

    /// 下書き記事を更新
    #[instrument(skip(pool))]
    pub async fn update(
        pool: &PgPool,
        article_id: Uuid,
        title: &str,
        slug: &str,
        body: &str,
        description: Option<&str>,
    ) -> Result<(), CmsError> {
        DraftArticleRepository::update(pool, article_id, title, slug, body, description, utc_now())
            .await
    }

    /// 下書きを公開（draft_articles → published_articles に移動）
    #[instrument(skip(pool))]
    pub async fn publish(pool: &PgPool, draft_id: Uuid) -> Result<Uuid, CmsError> {
        let draft = DraftArticleQuery::fetch_by_id(pool, draft_id)
            .await?
            .ok_or(CmsError::NotFound)?;

        // スラッグのバリデーション
        let slug = PublishedArticleSlug::new(draft.article.slug.clone())?;

        // スラッグ重複チェック（新規公開なので除外IDなし）
        if PublishedArticleQuery::exists_by_slug(pool, slug.as_str(), None).await? {
            return Err(CmsError::ValidationError(
                "このスラッグは既に使用されています".to_string(),
            ));
        }

        let now = utc_now();

        // 公開記事を作成
        let published_id = PublishedArticleRepository::create_from_draft(pool, &draft, now).await?;

        // 下書きを削除
        DraftArticleRepository::delete(pool, draft_id).await?;

        Ok(published_id)
    }

    /// 下書き記事を削除
    #[instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, article_id: Uuid) -> Result<(), CmsError> {
        DraftArticleRepository::delete(pool, article_id).await
    }
}

/// 管理画面用: 全記事一覧サービス
#[derive(Debug, Clone)]
pub struct AdminArticleService;

impl AdminArticleService {
    /// 公開記事と下書き記事の統合一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<ArticleListItem>, CmsError> {
        AdminArticleQuery::fetch_all(pool).await
    }
}
