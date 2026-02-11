use crate::error::CmsError;
use crate::models::Category;
use crate::queries::CategoryQuery;
use crate::repositories::CategoryRepository;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

/// カテゴリサービス
#[derive(Debug, Clone)]
pub struct CategoryService {
    pool: PgPool,
}

impl CategoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 全カテゴリを取得
    #[instrument(skip(self))]
    pub async fn fetch_all(&self) -> Result<Vec<Category>, CmsError> {
        CategoryQuery::fetch_all(&self.pool).await
    }

    /// 下書き記事のカテゴリを名前リストで保存
    #[instrument(skip(self))]
    pub async fn save_for_draft(&self, article_id: Uuid, names: &[String]) -> Result<(), CmsError> {
        let mut ids = Vec::with_capacity(names.len());
        for name in names {
            let cat = CategoryRepository::find_or_create_by_name(&self.pool, name).await?;
            ids.push(cat.id);
        }
        CategoryRepository::replace_for_draft(&self.pool, article_id, &ids).await
    }

    /// 公開記事のカテゴリを名前リストで保存
    #[instrument(skip(self))]
    pub async fn save_for_published(
        &self,
        article_id: Uuid,
        names: &[String],
    ) -> Result<(), CmsError> {
        let mut ids = Vec::with_capacity(names.len());
        for name in names {
            let cat = CategoryRepository::find_or_create_by_name(&self.pool, name).await?;
            ids.push(cat.id);
        }
        CategoryRepository::replace_for_published(&self.pool, article_id, &ids).await
    }
}
