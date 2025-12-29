use crate::common::handlers::get_preview_article_handler;
use crate::common::response::set_article_page_cache_control;
use crate::front::components::article_detail::ArticleDetail;
use crate::front::components::header::Header;
use crate::front::components::not_found::NotFound;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use std::error::Error;

use super::PreviewArticlePageMeta;

#[component]
pub(crate) fn PreviewArticlePage() -> impl IntoView {
    set_article_page_cache_control();

    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();

    let article = Resource::new(id, move |id| async move {
        get_preview_article_handler(id).await
    });

    view! {
        <Header is_h1=false />
        <Suspense fallback=|| {
            "Loading..."
        }>
            {move || {
                article
                    .map(|article| {
                        match article {
                            Ok(Some(article)) => {
                                view! {
                                    <PreviewArticlePageMeta meta=article.article_meta_dto.clone() />
                                    <ArticleDetail article=article.article_detail_dto.clone() />
                                    <script>{"hljs.highlightAll();"}</script>
                                    // newt embed
                                    <script async src="//cdn.iframe.ly/embed.js"></script>
                                }
                                    .into_any()
                            }
                            Ok(None) => view! { <NotFound /> }.into_any(),
                            Err(e) => {
                                view! { <p>{format!("Error: {:?}", e.source())}</p> }.into_any()
                            }
                        }
                    })
            }}
        </Suspense>
    }
}
