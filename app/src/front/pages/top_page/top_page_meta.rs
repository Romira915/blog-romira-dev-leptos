use crate::constants::{ORIGIN, WEB_APP_DESCRIPTION, WEB_APP_TITLE, WEB_TOP_PAGE_OG_IMAGE_URL};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[component]
pub(crate) fn TopPageMeta() -> impl IntoView {
    view! {
        <Title text=WEB_APP_TITLE />
        <Meta name="description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:title" content=WEB_APP_TITLE />
        <Meta property="og:description" content=WEB_APP_DESCRIPTION />
        <Meta property="og:type" content="website" />
        <Meta property="og:url" content=ORIGIN />
        <Meta property="og:site_name" content=WEB_APP_TITLE />
        <Meta property="og:image" content=WEB_TOP_PAGE_OG_IMAGE_URL />
        <Meta name="twitter:creator" content="@Romira915" />
    }
}
