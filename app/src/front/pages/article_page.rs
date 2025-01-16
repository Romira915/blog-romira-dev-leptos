use crate::common::handlers::{get_article_handler, get_articles_handler};
use crate::error::GetArticleError;
use crate::front::components::not_found::NotFound;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use std::error::Error;
use std::sync::Arc;
use stylance::import_style;

import_style!(pub(crate) article_page_style, "article_page.module.scss");

#[component]
pub(crate) fn ArticlePage() -> impl IntoView {
    let params = use_params_map();
    let id = Arc::new(params.read().get("id").unwrap_or_default());

    let article = Resource::new(
        || (),
        move |_| {
            let id = id.clone();
            async move { get_article_handler(id).await }
        },
    );

    view! {
        <Suspense fallback=|| {
            "Loading..."
        }>
            {move || {
                article
                    .map(|article| {
                        match article {
                            Ok(Some(article)) => view! { <div>{article.title}</div> }.into_any(),
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
