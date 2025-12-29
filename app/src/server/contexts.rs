use crate::constants::{
    NEWT_BASE_URL, NEWT_CDN_BASE_URL, PRTIMES_WORD_PRESS_BASE_URL, QIITA_BASE_URL,
};
use crate::server::services::newt::NewtArticleService;
use crate::server::services::qiita::QiitaArticleService;
use crate::server::services::word_press::WordPressArticleService;
use axum::extract::FromRef;
use blog_romira_dev_cms::{AdminArticleService, DraftArticleService, PublishedArticleService};
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
}

impl AppState {
    #[instrument(skip(db_pool))]
    pub fn new(leptos_options: LeptosOptions, db_pool: PgPool) -> Self {
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
            admin_article_service: AdminArticleService::new(db_pool),
        }
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
}
