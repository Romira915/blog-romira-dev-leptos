use crate::common::dto::ArticleMetaDto;
use crate::common::handlers::get_article_handler;
use crate::constants::{ORIGIN, WEB_APP_TITLE};
use crate::front::components::article_detail::ArticleDetail;
use crate::front::components::header::Header;
use crate::front::components::not_found::NotFound;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
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
                                    <ArticlePageMeta meta=article.article_meta_dto.clone() />
                                    <ArticleDetail article=article.article_detail_dto.clone() />
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

#[component]
pub(crate) fn ArticlePageMeta(meta: ArticleMetaDto) -> impl IntoView {
    let keywords = meta
        .keywords
        .iter()
        .map(|k| k.get_untracked())
        .collect::<Vec<String>>()
        .join(", ");
    view! {
        <Title text=meta.title.get() />
        <Meta name="description" content=meta.description.get_untracked() />
        <Meta name="keywords" content=keywords />
        <Meta name="date" content=meta.published_at.get_untracked() />
        <Meta name="creation_date" content=meta.first_published_at.get_untracked() />
        <Meta property="og:sitename" content=WEB_APP_TITLE />
        <Meta property="og:title" content=meta.title.get_untracked() />
        <Meta property="og:description" content=meta.description.get_untracked() />
        <Meta property="og:image" content=meta.og_image_url.get_untracked() />
        <Meta property="og:type" content="article" />
        <Meta
            property="og:url"
            content=format!("{}/articles/{}", ORIGIN, meta.id.get_untracked())
        />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=meta.title.get_untracked() />
        <Meta name="twitter:description" content=meta.description.get_untracked() />
        <Meta name="twitter:image" content=meta.og_image_url.get_untracked() />
        <Meta name="twitter:creator" content="@Romira915" />
    }
}
