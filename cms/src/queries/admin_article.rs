use crate::error::CmsError;
use crate::models::{
    ArticleListItem, Category, DraftArticle, DraftArticleWithCategories, PublishedArticle,
    PublishedArticleWithCategories,
};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 管理画面用記事クエリサービス（SELECT操作）
pub struct AdminArticleQuery;

impl AdminArticleQuery {
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
        result.sort_by_key(|a| std::cmp::Reverse(a.updated_at()));

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
