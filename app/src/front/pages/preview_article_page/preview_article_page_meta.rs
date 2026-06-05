use crate::common::dto::ArticleMetaDto;
use crate::constants::{ORIGIN, WEB_APP_TITLE};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[component]
pub(crate) fn PreviewArticlePageMeta(meta: ArticleMetaDto) -> impl IntoView {
    let keywords = meta.keywords.join(", ");
    view! {
        <Title text=meta.title.clone() />
        <Meta name="description" content=meta.description.clone() />
        <Meta name="keywords" content=keywords />
        <Meta name="date" content=meta.published_at.clone() />
        <Meta name="creation_date" content=meta.first_published_at.clone() />
        <Meta property="og:sitename" content=WEB_APP_TITLE />
        <Meta property="og:title" content=meta.title.clone() />
        <Meta property="og:description" content=meta.description.clone() />
        <Meta property="og:image" content=meta.og_image_url.clone() />
        <Meta property="og:type" content="article" />
        <Meta property="article:published_time" content=meta.published_at.clone() />
        <Meta property="og:url" content=format!("{}/articles/{}", ORIGIN, meta.slug) />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=meta.title />
        <Meta name="twitter:description" content=meta.description />
        <Meta name="twitter:image" content=meta.og_image_url />
        <Meta name="twitter:creator" content="@Romira915" />
    }
}
