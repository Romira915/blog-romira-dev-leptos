use crate::constants::{NEWT_BASE_URL, NEWT_CDN_BASE_URL};
use crate::server::services::NewtArticleService;
use axum::extract::FromRef;
use leptos::prelude::*;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub newt_article_service: NewtArticleService,
}

impl AppState {
    pub fn new(leptos_options: LeptosOptions) -> Self {
        Self {
            leptos_options,
            newt_article_service: NewtArticleService::new(
                reqwest::Client::new(),
                NEWT_CDN_BASE_URL,
                NEWT_BASE_URL,
            ),
        }
    }
}
