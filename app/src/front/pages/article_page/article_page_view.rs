use crate::common::dto::ArticleResponse;
use crate::common::handlers::get_article_handler;
use crate::common::response::set_article_page_cache_control;
use crate::front::components::article_detail::ArticleDetail;
use crate::front::components::header::Header;
use crate::front::components::not_found::NotFound;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map};

use super::ArticlePageMeta;

#[component]
pub(crate) fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    set_article_page_cache_control(&id());

    let article = Resource::new(id, move |id| async move { get_article_handler(id).await });

    view! {
        <Header is_h1=false />
        <Suspense fallback=|| {
            "Loading..."
        }>
            {move || {
                article
                    .map(|response| {
                        match response {
                            Ok(ArticleResponse::Found(article)) => {
                                view! {
                                    <ArticlePageMeta meta=article.article_meta_dto.clone() />
                                    <ArticleDetail article=article.article_detail_dto.clone() />
                                    <script>{"hljs.highlightAll();"}</script>
                                    // newt embed
                                    <script async src="//cdn.iframe.ly/embed.js"></script>
                                }
                                    .into_any()
                            }
                            Ok(ArticleResponse::Redirect(url)) => {
                                view! { <ClientRedirect url=url.clone() /> }.into_any()
                            }
                            Ok(ArticleResponse::NotFound(())) => view! { <NotFound /> }.into_any(),
                            Err(e) => view! { <p>{format!("Error: {}", e)}</p> }.into_any(),
                        }
                    })
            }}
        </Suspense>
    }
}

/// クライアントサイドリダイレクト（replace: trueで履歴を置き換え）
#[allow(clippy::unused_unit)]
#[component]
fn ClientRedirect(url: String) -> impl IntoView {
    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate(
            &url,
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
    });
    ()
}
