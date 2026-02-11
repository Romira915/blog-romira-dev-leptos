use crate::constants::{
    NEWT_BASE_URL, NEWT_CDN_BASE_URL, PRTIMES_WORD_PRESS_BASE_URL, QIITA_BASE_URL,
};
use crate::server::config::SERVER_CONFIG;
use crate::server::services::imgix::ImgixService;
use crate::server::services::newt::NewtArticleService;
use crate::server::services::qiita::QiitaArticleService;
use crate::server::services::signing::GcsSigningService;
use crate::server::services::word_press::WordPressArticleService;
use axum::extract::FromRef;
use blog_romira_dev_cms::{
    AdminArticleService, CategoryService, DraftArticleService, ImageService,
    PublishedArticleService,
};
use leptos::prelude::*;
use sqlx::PgPool;
use tracing::instrument;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub(crate) leptos_options: LeptosOptions,
    pub(crate) db_pool: PgPool,
    pub(crate) newt_article_service: NewtArticleService,
    pub(crate) word_press_article_service: WordPressArticleService,
    pub(crate) qiita_article_service: QiitaArticleService,
    pub(crate) published_article_service: PublishedArticleService,
    pub(crate) draft_article_service: DraftArticleService,
    pub(crate) admin_article_service: AdminArticleService,
    pub(crate) image_service: ImageService,
    pub(crate) category_service: CategoryService,
    pub(crate) signing_service: GcsSigningService,
    pub(crate) imgix_service: ImgixService,
}

impl AppState {
    #[instrument(skip(db_pool))]
    pub fn new(leptos_options: LeptosOptions, db_pool: PgPool) -> Self {
        let client = reqwest::Client::new();

        // 署名サービスの初期化
        let signing_service = GcsSigningService::from_service_account_key(
            SERVER_CONFIG.gcs_bucket.clone(),
            &SERVER_CONFIG.gcs_service_account_key_json,
        )
        .expect("Failed to initialize GCS signing service");

        // imgixサービスの初期化
        let imgix_service = ImgixService::new(SERVER_CONFIG.imgix_domain.clone());

        Self {
            leptos_options,
            db_pool: db_pool.clone(),
            newt_article_service: NewtArticleService::new(
                client.clone(),
                NEWT_CDN_BASE_URL,
                NEWT_BASE_URL,
            ),
            word_press_article_service: WordPressArticleService::new(
                client.clone(),
                PRTIMES_WORD_PRESS_BASE_URL,
            ),
            qiita_article_service: QiitaArticleService::new(client.clone(), QIITA_BASE_URL),
            published_article_service: PublishedArticleService::new(db_pool.clone()),
            draft_article_service: DraftArticleService::new(db_pool.clone()),
            admin_article_service: AdminArticleService::new(db_pool.clone()),
            image_service: ImageService::new(
                db_pool.clone(),
                SERVER_CONFIG.gcs_path_prefix.clone(),
            ),
            category_service: CategoryService::new(db_pool),
            signing_service,
            imgix_service,
        }
    }

    pub fn leptos_options(&self) -> &LeptosOptions {
        &self.leptos_options
    }

    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    pub fn published_article_service(&self) -> &PublishedArticleService {
        &self.published_article_service
    }

    pub fn draft_article_service(&self) -> &DraftArticleService {
        &self.draft_article_service
    }

    pub fn admin_article_service(&self) -> &AdminArticleService {
        &self.admin_article_service
    }

    pub fn image_service(&self) -> &ImageService {
        &self.image_service
    }

    pub fn signing_service(&self) -> &GcsSigningService {
        &self.signing_service
    }

    pub fn category_service(&self) -> &CategoryService {
        &self.category_service
    }

    pub fn imgix_service(&self) -> &ImgixService {
        &self.imgix_service
    }

    /// テスト用のインスタンスを作成（署名サービスはスタブ）
    #[cfg(any(test, feature = "test-utils"))]
    pub fn new_for_test(leptos_options: LeptosOptions, db_pool: PgPool) -> Self {
        let client = reqwest::Client::new();

        Self {
            leptos_options,
            db_pool: db_pool.clone(),
            newt_article_service: NewtArticleService::new(
                client.clone(),
                NEWT_CDN_BASE_URL,
                NEWT_BASE_URL,
            ),
            word_press_article_service: WordPressArticleService::new(
                client.clone(),
                PRTIMES_WORD_PRESS_BASE_URL,
            ),
            qiita_article_service: QiitaArticleService::new(client.clone(), QIITA_BASE_URL),
            published_article_service: PublishedArticleService::new(db_pool.clone()),
            draft_article_service: DraftArticleService::new(db_pool.clone()),
            admin_article_service: AdminArticleService::new(db_pool.clone()),
            image_service: ImageService::new(db_pool.clone(), "test".to_string()),
            category_service: CategoryService::new(db_pool),
            signing_service: GcsSigningService::new_stub("test-bucket".to_string()),
            imgix_service: ImgixService::new("test.imgix.net".to_string()),
        }
    }
}
