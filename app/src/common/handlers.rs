use crate::common::dto::HomePageArticleDto;
use crate::constants::{NEWT_BASE_URL, NEWT_CDN_BASE_URL};
use leptos::html::article;
use std::cmp::Reverse;

use crate::error::GetArticlesError;
use leptos::prelude::*;
use leptos::prelude::{ServerFnError, expect_context};
use reqwest::StatusCode;

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}

#[server]
pub(crate) async fn get_articles_handler()
-> Result<Vec<HomePageArticleDto>, ServerFnError<GetArticlesError>> {
    use crate::AppState;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let wordpress_article_service = app_state.word_press_article_service;
    let response = expect_context::<ResponseOptions>();

    let newt_articles = newt_article_service.get_published_articles().await;
    let newt_articles = match newt_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::from(GetArticlesError::from(err)));
        }
    };

    let wordpress_articles = wordpress_article_service.get_articles().await;
    let wordpress_articles = match wordpress_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::from(GetArticlesError::from(err)));
        }
    };

    let mut articles = newt_articles
        .items
        .into_iter()
        .map(HomePageArticleDto::from)
        .collect::<Vec<HomePageArticleDto>>();
    articles.extend(wordpress_articles.into_iter().map(HomePageArticleDto::from));
    articles.sort_unstable_by_key(|a| Reverse(a.published_at));

    Ok(articles)
}
