use crate::common::handlers::{get_article_handler, get_articles_handler};
use crate::error::GetArticleError;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use std::error::Error;
use std::sync::Arc;

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
                            Ok(None) => view! { <p>{"Not Found"}</p> }.into_any(),
                            Err(e) => {
                                view! { <p>{format!("Error: {:?}", e.source())}</p> }.into_any()
                            }
                        }
                    })
            }}
        </Suspense>
    }
}
