use crate::common::dto::{HomePageArticleDto, HomePageAuthorDto};
use crate::constants::{HOUR, JST_TZ, NEWT_BASE_URL, NEWT_CDN_BASE_URL, ROMIRA_NEWT_AUTHOR_ID};
use crate::error::GetArticlesError;
use chrono::{FixedOffset, TimeZone};
use leptos::html::article;
use leptos::prelude::*;
use leptos::prelude::{ServerFnError, expect_context};
use reqwest::StatusCode;
use std::cmp::Reverse;

#[server]
pub(crate) async fn get_number() -> Result<i32, ServerFnError> {
    tracing::info!("get_number");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(100)
}

#[server(endpoint = "get_articles_handler")]
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
            return Err(ServerFnError::from(
                GetArticlesError::NewtArticleServiceGetArticles(err.to_string()),
            ));
        }
    };

    let wordpress_articles = wordpress_article_service.get_articles().await;
    let wordpress_articles = match wordpress_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::from(
                GetArticlesError::WordPressArticleService(err.to_string()),
            ));
        }
    };

    let mut articles = newt_articles
        .items
        .into_iter()
        .map(HomePageArticleDto::from)
        .collect::<Vec<HomePageArticleDto>>();
    // #[cfg(debug_assertions)]
    // articles.push(HomePageArticleDto {
    //     title: "Debug Long Title".repeat(30).to_string(),
    //     thumbnail_url: "https://blog-romira.imgix.net/95424e09-0b44-4165-a5d9-498fdad10553/Windows・WSLの開発環境をansibleで管理しているという話.jpg".to_string(),
    //     src: format!("{}/articles/debug", NEWT_BASE_URL),
    //     category: vec!["Debug".to_string()],
    //     published_at: chrono::Utc::now()
    //         .with_timezone(&FixedOffset::east_opt(JST_TZ * HOUR).unwrap()),
    // });

    articles.extend(wordpress_articles.into_iter().map(HomePageArticleDto::from));
    articles.sort_unstable_by_key(|a| Reverse(a.published_at.get()));

    Ok(articles)
}

#[server(endpoint = "get_author_handler")]
pub(crate) async fn get_author_handler()
-> Result<HomePageAuthorDto, ServerFnError<GetArticlesError>> {
    use crate::AppState;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let response = expect_context::<ResponseOptions>();

    let author = newt_article_service.get_author(ROMIRA_NEWT_AUTHOR_ID).await;
    let author = match author {
        Ok(author) => author,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::from(
                GetArticlesError::NewtArticleServiceGetAuthor(err.to_string()),
            ));
        }
    };

    Ok(author.into())
}
