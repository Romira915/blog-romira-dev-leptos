use super::CategoryQuery;
use crate::error::CmsError;
use crate::models::{
    ArticleListItem, DraftArticle, DraftArticleWithCategories, PublishedArticle,
    PublishedArticleWithCategories,
};
use sqlx::PgPool;
use tracing::instrument;

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
            let categories = CategoryQuery::fetch_for_published(pool, article.id).await?;
            result.push(ArticleListItem::Published(PublishedArticleWithCategories {
                article,
                categories,
            }));
        }

        // 下書き記事を追加
        for article in drafts {
            let categories = CategoryQuery::fetch_for_draft(pool, article.id).await?;
            result.push(ArticleListItem::Draft(DraftArticleWithCategories {
                article,
                categories,
            }));
        }

        // 作成日時でソート（新しい順）
        result.sort_by_key(|a| std::cmp::Reverse(a.created_at()));

        Ok(result)
    }
}

//noinspection NonAsciiCharacters
//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use chrono::NaiveDateTime;

    fn parse_datetime(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[sqlx::test]
    async fn test_公開記事のみの場合fetch_allで公開記事が取得されること(
        pool: PgPool,
    ) {
        let published_id = insert_published_article(
            &pool,
            "pub-slug",
            "Published",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], ArticleListItem::Published(_)));
        assert_eq!(result[0].id(), published_id);
    }

    #[sqlx::test]
    async fn test_下書きのみの場合fetch_allで下書きが取得されること(
        pool: PgPool,
    ) {
        let draft_id = insert_draft_article(&pool, "draft-slug", "Draft", "Body", None).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], ArticleListItem::Draft(_)));
        assert_eq!(result[0].id(), draft_id);
    }

    #[sqlx::test]
    async fn test_fetch_allで公開と下書き両方が取得されること(pool: PgPool) {
        insert_published_article(
            &pool,
            "pub-slug",
            "Published",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;
        insert_draft_article(&pool, "draft-slug", "Draft", "Body", None).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 2);

        let has_published = result
            .iter()
            .any(|a| matches!(a, ArticleListItem::Published(_)));
        let has_draft = result
            .iter()
            .any(|a| matches!(a, ArticleListItem::Draft(_)));

        assert!(has_published);
        assert!(has_draft);
    }

    #[sqlx::test]
    async fn test_fetch_allで未来の公開記事も取得されること(pool: PgPool) {
        let future_id = insert_published_article(
            &pool,
            "future-slug",
            "Future Article",
            "Body",
            None,
            parse_datetime("2099-01-01 10:00:00"),
        )
        .await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), future_id);
    }

    #[sqlx::test]
    async fn test_fetch_allで公開記事のカテゴリも取得されること(pool: PgPool) {
        let cat_id = create_test_category(&pool, "PubCat", "pubcat").await;
        let article_id = insert_published_article(
            &pool,
            "cat-pub",
            "Categorized Published",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;

        link_published_article_category(&pool, article_id, cat_id).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        let test_article = result
            .iter()
            .find(|a| a.id() == article_id)
            .expect("Article not found");

        match test_article {
            ArticleListItem::Published(p) => {
                assert_eq!(p.categories.len(), 1);
            }
            _ => panic!("Expected Published article"),
        }
    }

    #[sqlx::test]
    async fn test_fetch_allで下書きのカテゴリも取得されること(pool: PgPool) {
        let cat_id = create_test_category(&pool, "DraftCat", "draftcat").await;
        let article_id =
            insert_draft_article(&pool, "cat-draft", "Categorized Draft", "Body", None).await;

        link_draft_article_category(&pool, article_id, cat_id).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        let test_article = result
            .iter()
            .find(|a| a.id() == article_id)
            .expect("Article not found");

        match test_article {
            ArticleListItem::Draft(d) => {
                assert_eq!(d.categories.len(), 1);
            }
            _ => panic!("Expected Draft article"),
        }
    }

    #[sqlx::test]
    async fn test_is_draftで公開と下書きが正しく判定されること(pool: PgPool) {
        let published_id = insert_published_article(
            &pool,
            "pub",
            "Published",
            "Body",
            None,
            parse_datetime("2020-01-01 10:00:00"),
        )
        .await;
        let draft_id = insert_draft_article(&pool, "draft", "Draft", "Body", None).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        let published = result.iter().find(|a| a.id() == published_id).unwrap();
        let draft = result.iter().find(|a| a.id() == draft_id).unwrap();

        assert!(!published.is_draft());
        assert!(draft.is_draft());
    }

    #[sqlx::test]
    async fn test_published_atで公開日時が正しく取得されること(pool: PgPool) {
        let publish_time = parse_datetime("2025-06-15 14:30:00");
        let published_id =
            insert_published_article(&pool, "pub", "Published", "Body", None, publish_time).await;
        let draft_id = insert_draft_article(&pool, "draft", "Draft", "Body", None).await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        let published = result.iter().find(|a| a.id() == published_id).unwrap();
        let draft = result.iter().find(|a| a.id() == draft_id).unwrap();

        assert_eq!(published.published_at(), Some(publish_time));
        assert_eq!(draft.published_at(), None);
    }

    #[sqlx::test]
    async fn test_fetch_allで作成日時の降順にソートされること(pool: PgPool) {
        let t1 = parse_datetime("2020-01-01 10:00:00");
        let t2 = parse_datetime("2021-06-01 10:00:00");
        let t3 = parse_datetime("2023-03-15 10:00:00");
        let t4 = parse_datetime("2025-06-01 10:00:00");

        // 最古の公開記事
        let id_t1 = insert_published_article_with_created_at(
            &pool,
            "slug-t1",
            "Article T1",
            "Body",
            None,
            t1,
            t1,
        )
        .await;

        // 2番目に古い下書き記事
        let id_t2 =
            insert_draft_article_with_created_at(&pool, "slug-t2", "Article T2", "Body", None, t2)
                .await;

        // 3番目の公開記事
        let id_t3 = insert_published_article_with_created_at(
            &pool,
            "slug-t3",
            "Article T3",
            "Body",
            None,
            t3,
            t3,
        )
        .await;

        // 最新の下書き記事
        let id_t4 =
            insert_draft_article_with_created_at(&pool, "slug-t4", "Article T4", "Body", None, t4)
                .await;

        let result = AdminArticleQuery::fetch_all(&pool)
            .await
            .expect("Failed to fetch all");

        assert_eq!(result.len(), 4);
        // 作成日時の降順に並ぶこと
        assert_eq!(result[0].id(), id_t4);
        assert_eq!(result[1].id(), id_t3);
        assert_eq!(result[2].id(), id_t2);
        assert_eq!(result[3].id(), id_t1);
    }
}
