use crate::constants::{
    NEWT_BASE_URL, NEWT_CDN_BASE_URL, PRTIMES_WORD_PRESS_BASE_URL, QIITA_BASE_URL,
};
use crate::server::services::newt::NewtArticleService;
use crate::server::services::qiita::QiitaArticleService;
use crate::server::services::word_press::WordPressArticleService;
use axum::extract::FromRef;
use leptos::prelude::*;
use tracing::instrument;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub(crate) leptos_options: LeptosOptions,
    pub(crate) newt_article_service: NewtArticleService,
    pub(crate) word_press_article_service: WordPressArticleService,
    pub(crate) qiita_article_service: QiitaArticleService,
}

impl AppState {
    #[instrument]
    pub fn new(leptos_options: LeptosOptions) -> Self {
        let client = reqwest::Client::new();

        Self {
            leptos_options,
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
        }
    }
}
