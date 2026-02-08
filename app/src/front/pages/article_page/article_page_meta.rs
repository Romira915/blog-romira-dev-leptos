use crate::common::dto::ArticleMetaDto;
use crate::constants::{ORIGIN, WEB_APP_TITLE};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

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
        <Meta property="article:published_time" content=meta.published_at.get_untracked() />
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
