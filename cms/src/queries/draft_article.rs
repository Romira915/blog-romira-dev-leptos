use super::CategoryQuery;
use crate::error::CmsError;
use crate::models::{DraftArticle, DraftArticleWithCategories};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// 下書き記事クエリサービス（SELECT操作）
pub struct DraftArticleQuery;

impl DraftArticleQuery {
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
            let categories = CategoryQuery::fetch_for_draft(pool, article.id).await?;
            result.push(DraftArticleWithCategories {
                article,
                categories,
            });
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
                let categories = CategoryQuery::fetch_for_draft(pool, article.id).await?;
                Ok(Some(DraftArticleWithCategories {
                    article,
                    categories,
                }))
            }
            None => Ok(None),
        }
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Utc};

    fn utc_now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    async fn create_test_category(pool: &PgPool, name: &str, slug: &str) -> Uuid {
        sqlx::query_scalar!(
            r#"INSERT INTO categories (name, slug) VALUES ($1, $2) RETURNING id"#,
            name,
            slug
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test category")
    }

    async fn insert_draft_article(
        pool: &PgPool,
        slug: &str,
        title: &str,
        body: &str,
        description: Option<&str>,
    ) -> Uuid {
        let now = utc_now();
        sqlx::query_scalar!(
            r#"INSERT INTO draft_articles (slug, title, body, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5) RETURNING id"#,
            slug,
            title,
            body,
            description,
            now as _
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert draft article")
    }

    async fn link_draft_article_category(pool: &PgPool, article_id: Uuid, category_id: Uuid) {
        sqlx::query!(
            "INSERT INTO draft_article_categories (article_id, category_id) VALUES ($1, $2)",
            article_id,
            category_id
        )
        .execute(pool)
        .await
        .expect("Failed to link draft article category");
    }

    #[sqlx::test]
    async fn test_fetch_allでカテゴリも取得されること(pool: PgPool) {
        let cat1_id = create_test_category(&pool, "Category1", "category1").await;
        let cat2_id = create_test_category(&pool, "Category2", "category2").await;
        let article_id = insert_draft_article(&pool, "test-slug", "Title", "Body", None).await;

        link_draft_article_category(&pool, article_id, cat1_id).await;
        link_draft_article_category(&pool, article_id, cat2_id).await;

        let fetched = DraftArticleQuery::fetch_by_id(&pool, article_id)
            .await
            .expect("Failed to fetch")
            .expect("Article not found");

        assert_eq!(fetched.categories.len(), 2);
    }

    #[sqlx::test]
    async fn test_fetch_by_idでカテゴリ付き記事が取得されること(pool: PgPool) {
        let cat_id = create_test_category(&pool, "TestCat", "testcat").await;
        let article_id = insert_draft_article(
            &pool,
            "test-slug",
            "Test Title",
            "Test Body",
            Some("Test Description"),
        )
        .await;

        link_draft_article_category(&pool, article_id, cat_id).await;

        let result = DraftArticleQuery::fetch_by_id(&pool, article_id)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_some());
        let article_with_cats = result.unwrap();

        assert_eq!(article_with_cats.article.id, article_id);
        assert_eq!(article_with_cats.article.slug, "test-slug");
        assert_eq!(article_with_cats.article.title, "Test Title");
        assert_eq!(article_with_cats.categories.len(), 1);
    }

    #[sqlx::test]
    async fn test_存在しないidでfetch_by_idするとnoneが返ること(pool: PgPool) {
        let nonexistent_id = Uuid::now_v7();

        let result = DraftArticleQuery::fetch_by_id(&pool, nonexistent_id)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn test_fetch_by_idでカテゴリなし記事が取得されること(pool: PgPool) {
        let article_id =
            insert_draft_article(&pool, "no-cat-slug", "No Cat Title", "Body", None).await;

        let result = DraftArticleQuery::fetch_by_id(&pool, article_id)
            .await
            .expect("Failed to fetch by id");

        assert!(result.is_some());
        let article_with_cats = result.unwrap();

        assert_eq!(article_with_cats.article.id, article_id);
        assert!(article_with_cats.categories.is_empty());
    }
}
