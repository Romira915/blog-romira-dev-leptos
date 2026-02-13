use crate::constants::{
    ORIGIN, ROMIRA_GITHUB_URL, WEB_APP_DESCRIPTION, WEB_APP_TITLE, WEB_TOP_PAGE_OG_IMAGE_URL,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Script, Title};

#[component]
pub(crate) fn TopPageMeta() -> impl IntoView {
    let jsonld = format!(
        r#"{{"@context":"https://schema.org","@type":"WebSite","name":"{}","url":"{}","description":"{}","author":{{"@type":"Person","name":"Romira","url":"{}"}},"inLanguage":"ja"}}"#,
        WEB_APP_TITLE, ORIGIN, WEB_APP_DESCRIPTION, ROMIRA_GITHUB_URL
    );
    view! {
        <Title text=WEB_APP_TITLE />
        <Link rel="canonical" href=ORIGIN />
        <Script type_="application/ld+json">{jsonld}</Script>
        <Meta name="description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:title" content=WEB_APP_TITLE />
        <Meta property="og:description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:type" content="website" />
        <Meta property="og:url" content=ORIGIN />
        <Meta property="og:site_name" content=WEB_APP_TITLE />
        <Meta property="og:locale" content="ja_JP" />
        <Meta property="og:image" content=WEB_TOP_PAGE_OG_IMAGE_URL />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:site" content="@Romira915" />
        <Meta name="twitter:creator" content="@Romira915" />
        <Meta name="twitter:title" content=WEB_APP_TITLE />
        <Meta name="twitter:description" content=WEB_APP_DESCRIPTION />
        <Meta name="twitter:image" content=WEB_TOP_PAGE_OG_IMAGE_URL />
    }
}
