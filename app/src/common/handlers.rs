pub mod admin;
pub mod auth;

use crate::common::dto::{ArticlePageDto, ArticleResponse, HomePageArticleDto, HomePageAuthorDto};
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
    use crate::common::response::set_top_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let wordpress_article_service = app_state.word_press_article_service;
    let qiita_article_service = app_state.qiita_article_service;
    let published_article_service = app_state.published_article_service;
    let response = expect_context::<ResponseOptions>();

    // キャッシュコントロールを設定（既に設定済みならスキップ）
    set_top_page_cache_control();

    let mut articles = Vec::new();

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

    articles.extend(wordpress_articles.into_iter().map(HomePageArticleDto::from));
    articles.extend(qiita_articles.into_iter().map(HomePageArticleDto::from));

    match published_article_service.fetch_all().await {
        Ok(local_articles) => {
            articles.extend(local_articles.into_iter().map(HomePageArticleDto::from));
        }
        Err(err) => {
            tracing::warn!(error = err.to_string(), "Failed to get local articles");
        }
    }

    articles.sort_unstable_by_key(|a| Reverse(a.first_published_at.get()));

    Ok(articles)
}

#[instrument]
#[server(input = GetUrl, endpoint = "get_author_handler")]
pub(crate) async fn get_author_handler() -> Result<HomePageAuthorDto, ServerFnError<GetAuthorError>>
{
    use crate::AppState;
    use crate::common::response::set_top_page_cache_control;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let newt_article_service = app_state.newt_article_service;
    let response = expect_context::<ResponseOptions>();

    // キャッシュコントロールを設定（既に設定済みならスキップ）
    set_top_page_cache_control();

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
) -> Result<ArticleResponse, ServerFnError<GetArticleError>> {
    use crate::AppState;
    use crate::common::dto::ArticleResponse;
    use crate::common::response::set_article_page_cache_control;
    use crate::constants::get_newt_redirect_slug;
    use leptos_axum::ResponseOptions;

    let app_state = expect_context::<AppState>();
    let published_article_service = app_state.published_article_service;
    let response = expect_context::<ResponseOptions>();

    // キャッシュコントロールを設定
    set_article_page_cache_control(&id);

    // 1. DB記事をslugで検索
    match published_article_service.fetch_by_slug(&id).await {
        Ok(Some(article)) => {
            return Ok(ArticleResponse::Found(ArticlePageDto::from(article)));
        }
        Ok(None) => {
            // DB記事が見つからない場合、リダイレクトマッピングを確認
        }
        Err(err) => {
            response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(error = err.to_string(), "Failed to get article from DB");
            return Err(ServerFnError::from(GetArticleError::DatabaseError(
                "Failed to get article from DB".to_string(),
            )));
        }
    }

    // 2. Newtリダイレクトマッピングを確認 → 対応するDB記事にリダイレクト
    if let Some(slug) = get_newt_redirect_slug(&id) {
        let redirect_url = format!("/articles/{}", slug);

        // SSR時（Accept: text/html）のみ301リダイレクト
        if crate::server::http::request::is_ssr_request().await {
            response.set_status(StatusCode::MOVED_PERMANENTLY);
            response.insert_header(
                axum::http::header::LOCATION,
                axum::http::HeaderValue::from_str(&redirect_url).unwrap(),
            );
        }

        // クライアントナビゲーション時：ClientRedirectで処理
        return Ok(ArticleResponse::Redirect(redirect_url));
    }

    // 3. 見つからない場合は404
    response.set_status(StatusCode::NOT_FOUND);
    Ok(ArticleResponse::NotFound(()))
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
