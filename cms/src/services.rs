use crate::error::CmsError;
use crate::models::{
    ArticleListItem, Category, DraftArticle, DraftArticleWithCategories, PublishedArticle,
    PublishedArticleWithCategories,
};
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
        let now = utc_now();
        let articles = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE published_at <= $1
            ORDER BY published_at DESC
            "#,
            now as _
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(articles.len());
        for article in articles {
            let categories = self.fetch_categories(article.id).await?;
            result.push(PublishedArticleWithCategories { article, categories });
        }

        Ok(result)
    }

    /// 公開済み記事をIDで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_id(
        &self,
        article_id: Uuid,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        let now = utc_now();
        let article = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE id = $1 AND published_at <= $2
            "#,
            article_id,
            now as _
        )
        .fetch_optional(&self.pool)
        .await?;

        match article {
            Some(article) => {
                let categories = self.fetch_categories(article.id).await?;
                Ok(Some(PublishedArticleWithCategories { article, categories }))
            }
            None => Ok(None),
        }
    }

    /// 公開済み記事をslugで取得
    #[instrument(skip(self))]
    pub async fn fetch_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<PublishedArticleWithCategories>, CmsError> {
        let now = utc_now();
        let article = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            WHERE slug = $1 AND published_at <= $2
            "#,
            slug,
            now as _
        )
        .fetch_optional(&self.pool)
        .await?;

        match article {
            Some(article) => {
                let categories = self.fetch_categories(article.id).await?;
                Ok(Some(PublishedArticleWithCategories { article, categories }))
            }
            None => Ok(None),
        }
    }

    /// 公開記事のカテゴリを取得
    #[instrument(skip(self))]
    async fn fetch_categories(&self, article_id: Uuid) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN published_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }
}

/// 下書き記事サービス（管理画面用）
#[derive(Debug, Clone)]
pub struct DraftArticleService;

impl DraftArticleService {
    /// 下書き記事一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<DraftArticleWithCategories>, CmsError> {
        let articles = sqlx::query_as!(
            DraftArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   created_at as "created_at: _", updated_at as "updated_at: _"
            FROM draft_articles
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(articles.len());
        for article in articles {
            let categories = Self::fetch_categories(pool, article.id).await?;
            result.push(DraftArticleWithCategories { article, categories });
        }

        Ok(result)
    }

    /// 下書き記事をIDで取得
    #[instrument(skip(pool))]
    pub async fn fetch_by_id(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Option<DraftArticleWithCategories>, CmsError> {
        let article = sqlx::query_as!(
            DraftArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   created_at as "created_at: _", updated_at as "updated_at: _"
            FROM draft_articles
            WHERE id = $1
            "#,
            article_id
        )
        .fetch_optional(pool)
        .await?;

        match article {
            Some(article) => {
                let categories = Self::fetch_categories(pool, article.id).await?;
                Ok(Some(DraftArticleWithCategories { article, categories }))
            }
            None => Ok(None),
        }
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
        let now = utc_now();

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO draft_articles (slug, title, body, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING id
            "#,
            slug,
            title,
            body,
            description,
            now as _
        )
        .fetch_one(pool)
        .await?;

        Ok(id)
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
        let now = utc_now();

        let rows = sqlx::query!(
            r#"
            UPDATE draft_articles
            SET title = $1, slug = $2, body = $3, description = $4, updated_at = $5
            WHERE id = $6
            "#,
            title,
            slug,
            body,
            description,
            now as _,
            article_id
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(CmsError::NotFound);
        }

        Ok(())
    }

    /// 下書きを公開（draft_articles → published_articles に移動）
    #[instrument(skip(pool))]
    pub async fn publish(pool: &PgPool, draft_id: Uuid) -> Result<Uuid, CmsError> {
        let draft = Self::fetch_by_id(pool, draft_id)
            .await?
            .ok_or(CmsError::NotFound)?;

        let now = utc_now();

        // 新規公開記事を作成
        let published_id = sqlx::query_scalar!(
            r#"
            INSERT INTO published_articles (slug, title, body, description, cover_image_url, published_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6, $6)
            RETURNING id
            "#,
            &draft.article.slug,
            &draft.article.title,
            &draft.article.body,
            draft.article.description.as_deref(),
            draft.article.cover_image_url.as_deref(),
            now as _
        )
        .fetch_one(pool)
        .await?;

        // カテゴリをコピー
        for category in &draft.categories {
            sqlx::query!(
                "INSERT INTO published_article_categories (article_id, category_id) VALUES ($1, $2)",
                published_id,
                category.id
            )
            .execute(pool)
            .await?;
        }

        // 下書きを削除
        sqlx::query!("DELETE FROM draft_articles WHERE id = $1", draft_id)
            .execute(pool)
            .await?;

        Ok(published_id)
    }

    /// 下書き記事を削除
    #[instrument(skip(pool))]
    pub async fn delete(pool: &PgPool, article_id: Uuid) -> Result<(), CmsError> {
        let rows = sqlx::query!("DELETE FROM draft_articles WHERE id = $1", article_id)
            .execute(pool)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(CmsError::NotFound);
        }

        Ok(())
    }

    /// 下書き記事のカテゴリを取得
    #[instrument(skip(pool))]
    async fn fetch_categories(pool: &PgPool, article_id: Uuid) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN draft_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }
}

/// 管理画面用: 全記事一覧サービス
#[derive(Debug, Clone)]
pub struct AdminArticleService;

impl AdminArticleService {
    /// 公開記事と下書き記事の統合一覧を取得
    #[instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool) -> Result<Vec<ArticleListItem>, CmsError> {
        // 公開記事を取得
        let published = sqlx::query_as!(
            PublishedArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   published_at as "published_at: _", created_at as "created_at: _", updated_at as "updated_at: _"
            FROM published_articles
            ORDER BY published_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        // 下書き記事を取得
        let drafts = sqlx::query_as!(
            DraftArticle,
            r#"
            SELECT id, slug, title, body, description, cover_image_url,
                   created_at as "created_at: _", updated_at as "updated_at: _"
            FROM draft_articles
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(published.len() + drafts.len());

        // 公開記事を追加
        for article in published {
            let categories = Self::fetch_published_categories(pool, article.id).await?;
            result.push(ArticleListItem::Published(PublishedArticleWithCategories {
                article,
                categories,
            }));
        }

        // 下書き記事を追加
        for article in drafts {
            let categories = Self::fetch_draft_categories(pool, article.id).await?;
            result.push(ArticleListItem::Draft(DraftArticleWithCategories {
                article,
                categories,
            }));
        }

        // 更新日時でソート
        result.sort_by(|a, b| b.updated_at().cmp(&a.updated_at()));

        Ok(result)
    }

    #[instrument(skip(pool))]
    async fn fetch_published_categories(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN published_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    #[instrument(skip(pool))]
    async fn fetch_draft_categories(
        pool: &PgPool,
        article_id: Uuid,
    ) -> Result<Vec<Category>, CmsError> {
        let categories = sqlx::query_as!(
            Category,
            r#"
            SELECT c.id, c.name, c.slug
            FROM categories c
            INNER JOIN draft_article_categories ac ON c.id = ac.category_id
            WHERE ac.article_id = $1
            "#,
            article_id
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }
}
