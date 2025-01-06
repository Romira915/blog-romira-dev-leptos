use crate::common::dto::HomePageArticleDto;
use crate::constants::{NEWT_BASE_URL, NEWT_CDN_BASE_URL};

use leptos::prelude::*;
use leptos::prelude::{expect_context, ServerFnError};

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}

#[server]
pub(crate) async fn get_articles_handler()
    -> Result<Vec<HomePageArticleDto>, ServerFnError> {
    use crate::AppState;

    let newt_article_service = expect_context::<AppState>().newt_article_service;

    let articles = newt_article_service.get_published_articles().await?;
    let articles = articles.items.into_iter().map(HomePageArticleDto::from).collect();

    Ok(articles)
}
