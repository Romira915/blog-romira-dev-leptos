use crate::common::dto::ArticleMetaDto;
use crate::constants::{ORIGIN, ROMIRA_GITHUB_URL, WEB_APP_TITLE};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Script, Title};

#[component]
pub(crate) fn ArticlePageMeta(meta: ArticleMetaDto) -> impl IntoView {
    let keywords = meta
        .keywords
        .iter()
        .map(|k| k.get_untracked())
        .collect::<Vec<String>>()
        .join(", ");
    let canonical_url = format!("{}/articles/{}", ORIGIN, meta.id.get_untracked());
    let keywords_json = meta
        .keywords
        .iter()
        .map(|k| format!(r#""{}""#, k.get_untracked().replace('"', r#"\""#)))
        .collect::<Vec<String>>()
        .join(",");
    let jsonld = format!(
        r#"{{"@context":"https://schema.org","@type":"BlogPosting","headline":"{}","description":"{}","image":"{}","datePublished":"{}","dateModified":"{}","author":{{"@type":"Person","name":"Romira","url":"{}"}},"publisher":{{"@type":"Person","name":"Romira"}},"mainEntityOfPage":{{"@type":"WebPage","@id":"{}"}},"keywords":[{}],"inLanguage":"ja"}}"#,
        meta.title.get_untracked().replace('"', r#"\""#),
        meta.description.get_untracked().replace('"', r#"\""#),
        meta.og_image_url.get_untracked(),
        meta.first_published_at.get_untracked(),
        meta.published_at.get_untracked(),
        ROMIRA_GITHUB_URL,
        canonical_url,
        keywords_json
    );
    let article_tags = meta
        .keywords
        .iter()
        .map(|k| {
            let tag = k.get_untracked();
            view! { <Meta property="article:tag" content=tag /> }
        })
        .collect_view();
    view! {
        <Title text=meta.title.get() />
        <Link rel="canonical" href=canonical_url.clone() />
        <Script type_="application/ld+json">{jsonld}</Script>
        <Meta name="description" content=meta.description.get_untracked() />
        <Meta name="keywords" content=keywords />
        <Meta name="date" content=meta.published_at.get_untracked() />
        <Meta name="creation_date" content=meta.first_published_at.get_untracked() />
        <Meta property="og:site_name" content=WEB_APP_TITLE />
        <Meta property="og:title" content=meta.title.get_untracked() />
        <Meta property="og:description" content=meta.description.get_untracked() />
        <Meta property="og:image" content=meta.og_image_url.get_untracked() />
        <Meta property="og:type" content="article" />
        <Meta property="og:locale" content="ja_JP" />
        <Meta property="og:url" content=canonical_url />
        <Meta property="article:published_time" content=meta.first_published_at.get_untracked() />
        <Meta property="article:modified_time" content=meta.published_at.get_untracked() />
        <Meta property="article:author" content=ORIGIN />
        {article_tags}
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:site" content="@Romira915" />
        <Meta name="twitter:creator" content="@Romira915" />
        <Meta name="twitter:title" content=meta.title.get_untracked() />
        <Meta name="twitter:description" content=meta.description.get_untracked() />
        <Meta name="twitter:image" content=meta.og_image_url.get_untracked() />
    }
}
