use crate::error::CmsError;
use crate::models::ArticleListItem;
use crate::queries::AdminArticleQuery;
use sqlx::PgPool;
use tracing::instrument;

/// 管理画面用: 全記事一覧サービス
#[derive(Debug, Clone)]
pub struct AdminArticleService {
    pool: PgPool,
}

impl AdminArticleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 公開記事と下書き記事の統合一覧を取得
    #[instrument(skip(self))]
    pub async fn fetch_all(&self) -> Result<Vec<ArticleListItem>, CmsError> {
        AdminArticleQuery::fetch_all(&self.pool).await
    }
}
