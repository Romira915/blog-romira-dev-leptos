pub mod admin;
pub(crate) mod auth;

use crate::common::dto::{ArticlePageDto, HomePageArticleDto, HomePageAuthorDto};
use crate::constants::ROMIRA_NEWT_AUTHOR_ID;
use crate::error::{GetArticleError, GetArticlesError, GetAuthorError};
use leptos::prelude::*;
use leptos::prelude::{ServerFnError, expect_context};
use leptos::server_fn::codec::GetUrl;
use reqwest::StatusCode;
use std::cmp::Reverse;
use tracing::instrument;

#[instrument]
#[server(input = GetUrl, endpoint = "get_articles_handler")]
pub(crate) async fn get_articles_handler()
-> Result<Vec<HomePageArticleDto>, ServerFnError<GetArticlesError>> {
    use crate::AppState;
    use crate::server::http::response::set_top_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let wordpress_article_service = app_state.word_press_article_service;
    let qiita_article_service = app_state.qiita_article_service;
    let response = expect_context::<ResponseOptions>();

    set_top_page_cache_control(&response);

    let newt_articles = newt_article_service.fetch_published_articles().await;
    let newt_articles = match newt_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(
                error = err.to_string(),
                "Failed to get articles from NewtArticleService",
            );
            return Err(ServerFnError::from(
                GetArticlesError::NewtArticleServiceGetArticles(
                    "Failed to get articles from NewtArticleService".to_string(),
                ),
            ));
        }
    };

    let wordpress_articles = wordpress_article_service.fetch_articles().await;
    let wordpress_articles = match wordpress_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(
                error = err.to_string(),
                "Failed to get articles from WordPressArticleService",
            );
            return Err(ServerFnError::from(
                GetArticlesError::WordPressArticleService(
                    "Failed to get articles from WordPressArticleService".to_string(),
                ),
            ));
        }
    };

    let qiita_articles = qiita_article_service.fetch_articles().await;
    let qiita_articles = match qiita_articles {
        Ok(articles) => articles,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(
                error = err.to_string(),
                "Failed to get articles from QiitaArticleService",
            );
            return Err(ServerFnError::from(GetArticlesError::QiitaArticleService(
                "Failed to get articles from QiitaArticleService".to_string(),
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
    articles.sort_unstable_by_key(|a| Reverse(a.first_published_at.get()));

    Ok(articles)
}

#[instrument]
#[server(input = GetUrl, endpoint = "get_author_handler")]
pub(crate) async fn get_author_handler() -> Result<HomePageAuthorDto, ServerFnError<GetAuthorError>>
{
    use crate::AppState;
    use crate::server::http::response::set_top_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let response = expect_context::<ResponseOptions>();

    set_top_page_cache_control(&response);

    let author = newt_article_service
        .fetch_author(ROMIRA_NEWT_AUTHOR_ID)
        .await;
    let author = match author {
        Ok(author) => author,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(error = err.to_string(), "Failed to get author");
            return Err(ServerFnError::from(
                GetAuthorError::NewtArticleServiceGetAuthor("Failed to get author".to_string()),
            ));
        }
    };

    Ok(author.into())
}

#[instrument]
#[server(input = GetUrl, endpoint = "get_article_handler")]
pub(crate) async fn get_article_handler(
    id: String,
) -> Result<Option<ArticlePageDto>, ServerFnError<GetArticleError>> {
    use crate::AppState;
    use crate::server::http::response::set_article_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let response = expect_context::<ResponseOptions>();

    set_article_page_cache_control(&response);

    let article = newt_article_service.fetch_published_article(&id).await;
    let article = match article {
        Ok(article) => article,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(
                error = err.to_string(),
                "Failed to get article from NewtArticleService",
            );
            return Err(ServerFnError::from(
                GetArticleError::NewtArticleServiceGetArticle(
                    "Failed to get article from NewtArticleService".to_string(),
                ),
            ));
        }
    };

    if article.is_none() {
        response.set_status(StatusCode::NOT_FOUND);
    }

    Ok(article.map(ArticlePageDto::from))
}

#[instrument]
#[server(input = GetUrl, endpoint = "get_preview_article_handler")]
pub(crate) async fn get_preview_article_handler(
    id: String,
) -> Result<Option<ArticlePageDto>, ServerFnError<GetArticleError>> {
    use crate::AppState;
    use crate::server::http::response::set_preview_article_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let response = expect_context::<ResponseOptions>();

    set_preview_article_page_cache_control(&response);

    let article = newt_article_service.fetch_preview_article(&id).await;
    let article = match article {
        Ok(article) => article,
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(
                error = err.to_string(),
                "Failed to get article from NewtArticleService",
            );
            return Err(ServerFnError::from(
                GetArticleError::NewtArticleServiceGetArticle(
                    "Failed to get article from NewtArticleService".to_string(),
                ),
            ));
        }
    };

    if article.is_none() {
        response.set_status(StatusCode::NOT_FOUND);
    }

    Ok(article.map(ArticlePageDto::from))
}
