use crate::common::dto::ArticleMetaDto;
use crate::constants::{ORIGIN, ROMIRA_GITHUB_URL, WEB_APP_TITLE};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Script, Title};

#[component]
pub(crate) fn ArticlePageMeta(meta: ArticleMetaDto) -> impl IntoView {
    let keywords = meta.keywords.join(", ");
    let canonical_url = format!("{}/articles/{}", ORIGIN, meta.slug);
    let keywords_json = meta
        .keywords
        .iter()
        .map(|k| format!(r#"\"{}\""#, k.replace('"', r#"\""#)))
        .collect::<Vec<String>>()
        .join(",");
    let jsonld = format!(
        r#"{{"@context":"https://schema.org","@type":"BlogPosting","headline":"{}","description":"{}","image":"{}","datePublished":"{}","dateModified":"{}","author":{{"@type":"Person","name":"Romira","url":"{}"}},"publisher":{{"@type":"Person","name":"Romira"}},"mainEntityOfPage":{{"@type":"WebPage","@id":"{}"}},"keywords":[{}],"inLanguage":"ja"}}"#,
        meta.title.replace('"', r#"\""#),
        meta.description.replace('"', r#"\""#),
        meta.og_image_url,
        meta.first_published_at,
        meta.published_at,
        ROMIRA_GITHUB_URL,
        canonical_url,
        keywords_json
    );
    let article_tags = meta
        .keywords
        .iter()
        .map(|k| view! { <Meta property="article:tag" content=k.clone() /> })
        .collect_view();
    view! {
        <Title text=meta.title.clone() />
        <Link rel="canonical" href=canonical_url.clone() />
        <Script type_="application/ld+json">{jsonld}</Script>
        <Meta name="description" content=meta.description.clone() />
        <Meta name="keywords" content=keywords />
        <Meta name="date" content=meta.published_at.clone() />
        <Meta name="creation_date" content=meta.first_published_at.clone() />
        <Meta property="og:site_name" content=WEB_APP_TITLE />
        <Meta property="og:title" content=meta.title.clone() />
        <Meta property="og:description" content=meta.description.clone() />
        <Meta property="og:image" content=meta.og_image_url.clone() />
        <Meta property="og:type" content="article" />
        <Meta property="og:locale" content="ja_JP" />
        <Meta property="og:url" content=canonical_url />
        <Meta property="article:published_time" content=meta.first_published_at.clone() />
        <Meta property="article:modified_time" content=meta.published_at.clone() />
        <Meta property="article:author" content=ORIGIN />
        {article_tags}
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:site" content="@Romira915" />
        <Meta name="twitter:creator" content="@Romira915" />
        <Meta name="twitter:title" content=meta.title />
        <Meta name="twitter:description" content=meta.description />
        <Meta name="twitter:image" content=meta.og_image_url />
    }
}
