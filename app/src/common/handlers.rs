use crate::common::dto::{HomePageArticleDto, HomePageAuthorDto};
use crate::constants::ROMIRA_NEWT_AUTHOR_ID;
use crate::error::GetArticlesError;
use leptos::prelude::*;
use leptos::prelude::{ServerFnError, expect_context};
use reqwest::StatusCode;
use std::cmp::Reverse;

#[server(endpoint = "get_articles_handler")]
pub(crate) async fn get_articles_handler()
-> Result<Vec<HomePageArticleDto>, ServerFnError<GetArticlesError>> {
    use crate::AppState;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let wordpress_article_service = app_state.word_press_article_service;
    let qiita_article_service = app_state.qiita_article_service;
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

    let qiita_articles = qiita_article_service.get_articles().await;
    let qiita_articles = match qiita_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(ServerFnError::from(GetArticlesError::QiitaArticleService(
                err.to_string(),
            )));
        }
    };

    let mut articles = newt_articles
        .items
        .into_iter()
        .map(HomePageArticleDto::from)
        .collect::<Vec<HomePageArticleDto>>();

    articles.extend(wordpress_articles.into_iter().map(HomePageArticleDto::from));
    articles.extend(qiita_articles.into_iter().map(HomePageArticleDto::from));
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
