use crate::error::CmsError;
use crate::models::Category;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// カテゴリリポジトリ（CUD操作）
pub struct CategoryRepository;

impl CategoryRepository {
    /// 名前でカテゴリを検索し、なければ作成
    #[instrument(skip(pool))]
    pub async fn find_or_create_by_name(pool: &PgPool, name: &str) -> Result<Category, CmsError> {
        let slug = name.trim().to_lowercase().replace(' ', "-");
        let category = sqlx::query_as!(
            Category,
            r#"
            INSERT INTO categories (name, slug)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id, name, slug
            "#,
            name.trim(),
            slug
        )
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    /// 下書き記事のカテゴリを全置換（DELETE + INSERT）
    #[instrument(skip(pool))]
    pub async fn replace_for_draft(
        pool: &PgPool,
        article_id: Uuid,
        category_ids: &[Uuid],
    ) -> Result<(), CmsError> {
        sqlx::query!(
            "DELETE FROM draft_article_categories WHERE article_id = $1",
            article_id
        )
        .execute(pool)
        .await?;

        for &cat_id in category_ids {
            sqlx::query!(
                "INSERT INTO draft_article_categories (article_id, category_id) VALUES ($1, $2)",
                article_id,
                cat_id
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// 公開記事のカテゴリを全置換（DELETE + INSERT）
    #[instrument(skip(pool))]
    pub async fn replace_for_published(
        pool: &PgPool,
        article_id: Uuid,
        category_ids: &[Uuid],
    ) -> Result<(), CmsError> {
        sqlx::query!(
            "DELETE FROM published_article_categories WHERE article_id = $1",
            article_id
        )
        .execute(pool)
        .await?;

        for &cat_id in category_ids {
            sqlx::query!(
                "INSERT INTO published_article_categories (article_id, category_id) VALUES ($1, $2)",
                article_id,
                cat_id
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[sqlx::test]
    async fn test_find_or_create_by_nameで新規カテゴリが作成されること(
        pool: PgPool,
    ) {
        let category = CategoryRepository::find_or_create_by_name(&pool, "Rust")
            .await
            .expect("Failed to find_or_create");

        assert_eq!(category.name, "Rust");
        assert_eq!(category.slug, "rust");

        // DBに存在することを確認
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM categories WHERE name = 'Rust'"#
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to count");

        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn test_find_or_create_by_nameで既存カテゴリが返されること(pool: PgPool) {
        let cat_id = create_test_category(&pool, "Rust", "rust").await;

        let category = CategoryRepository::find_or_create_by_name(&pool, "Rust")
            .await
            .expect("Failed to find_or_create");

        assert_eq!(category.id, cat_id);
        assert_eq!(category.name, "Rust");
    }

    #[sqlx::test]
    async fn test_find_or_create_by_nameで前後の空白がトリムされること(
        pool: PgPool,
    ) {
        let category = CategoryRepository::find_or_create_by_name(&pool, "  Web Dev  ")
            .await
            .expect("Failed to find_or_create");

        assert_eq!(category.name, "Web Dev");
        assert_eq!(category.slug, "web-dev");
    }

    #[sqlx::test]
    async fn test_replace_for_draftでカテゴリが全置換されること(pool: PgPool) {
        let article_id = insert_draft_article(&pool, "test-slug", "テスト", "本文", None).await;
        let cat1_id = create_test_category(&pool, "Cat1", "cat1").await;
        let cat2_id = create_test_category(&pool, "Cat2", "cat2").await;
        let cat3_id = create_test_category(&pool, "Cat3", "cat3").await;

        // 初期リンク
        link_draft_article_category(&pool, article_id, cat1_id).await;
        link_draft_article_category(&pool, article_id, cat2_id).await;

        // cat2, cat3 に置換
        CategoryRepository::replace_for_draft(&pool, article_id, &[cat2_id, cat3_id])
            .await
            .expect("Failed to replace");

        let linked: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT category_id FROM draft_article_categories WHERE article_id = $1 ORDER BY category_id"#,
            article_id
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(linked.len(), 2);
        assert!(linked.contains(&cat2_id));
        assert!(linked.contains(&cat3_id));
        assert!(!linked.contains(&cat1_id));
    }

    #[sqlx::test]
    async fn test_replace_for_publishedでカテゴリが全置換されること(pool: PgPool) {
        let article_id =
            insert_published_article(&pool, "test-slug", "テスト", "本文", None, utc_now()).await;
        let cat1_id = create_test_category(&pool, "Cat1", "cat1").await;
        let cat2_id = create_test_category(&pool, "Cat2", "cat2").await;

        // 初期リンク
        link_published_article_category(&pool, article_id, cat1_id).await;

        // cat2 のみに置換
        CategoryRepository::replace_for_published(&pool, article_id, &[cat2_id])
            .await
            .expect("Failed to replace");

        let linked: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT category_id FROM published_article_categories WHERE article_id = $1"#,
            article_id
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch");

        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0], cat2_id);
    }

    #[sqlx::test]
    async fn test_replace_for_draftで空配列を渡すと全リンクが削除されること(
        pool: PgPool,
    ) {
        let article_id = insert_draft_article(&pool, "test-slug", "テスト", "本文", None).await;
        let cat_id = create_test_category(&pool, "Cat1", "cat1").await;
        link_draft_article_category(&pool, article_id, cat_id).await;

        CategoryRepository::replace_for_draft(&pool, article_id, &[])
            .await
            .expect("Failed to replace");

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM draft_article_categories WHERE article_id = $1"#,
            article_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to count");

        assert_eq!(count, 0);
    }
}
